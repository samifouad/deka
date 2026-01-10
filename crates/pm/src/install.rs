use crate::{
    bun_lock::BunLock,
    cache::{
        CachePaths, cache_key, compute_sha512, copy_package, download_tarball, extract_tarball,
    },
    lock,
    npm::{fetch_npm_metadata, resolve_package_version},
    payload::InstallPayload,
    spec::{Ecosystem, parse_hinted_spec, parse_package_spec},
};
use anyhow::{Context, Result, bail};
use serde_json::{Map, Value, json};
use std::{
    collections::{BTreeMap, HashSet, VecDeque},
    fs,
    io::Write,
    path::PathBuf,
    sync::Arc,
    time::Instant,
};

pub fn run_install(payload: InstallPayload) -> Result<()> {
    let mut specs = payload.specs.clone();
    let auto_resolved = specs.is_empty();
    if auto_resolved {
        specs = collect_project_specs()?;
    }

    let override_ecosystem = payload
        .ecosystem
        .as_deref()
        .and_then(Ecosystem::from_str)
        .unwrap_or(Ecosystem::Node);
    let quiet = payload.quiet;

    if payload.prompt && !payload.yes {
        let message = if auto_resolved {
            "Install dependencies listed in package.json? [y/N]:"
        } else {
            "Install requested dependencies? [y/N]:"
        };
        if !prompt_yes_no(message)? {
            bail!("installation aborted by user");
        }
    }

    let cache = CachePaths::new()?;
    cache.ensure()?;

    let bun_lock = BunLock::load()?;
    let mut ctx = InstallContext::new(cache, bun_lock.map(Arc::new));
    for spec in specs {
        ctx.enqueue(spec, None, false, None);
    }

    let start = Instant::now();

    while let Some(task) = ctx.next_task() {
        let sequence = ctx.completed_count + 1;
        emit_progress(ctx.scheduled_count(), ctx.completed_count, 1)?;
        let (ecosystem, spec_str) = parse_hinted_spec(&task.spec, Some(override_ecosystem))?;
        if ecosystem != Ecosystem::Node {
            if !quiet {
                eprintln!(
                    "helper    skipping unsupported ecosystem {} for {}",
                    ecosystem.as_str(),
                    spec_str
                );
            }
            ctx.completed_count += 1;
            emit_progress(
                ctx.scheduled_count(),
                ctx.completed_count,
                ctx.pending_count(),
            )?;
            continue;
        }

        emit_package(
            &spec_str,
            "fetching",
            ecosystem.as_str(),
            sequence,
            ctx.scheduled_count(),
            0,
        )?;

        match install_node_package(&mut ctx, &spec_str, task.lock_key.as_deref()) {
            Ok(result) => {
                for dep in result.dependencies {
                    let lock_key = dep.lock_key.clone();
                    ctx.enqueue(
                        dep.spec,
                        Some(dep.descriptor.clone()),
                        false,
                        Some(lock_key),
                    );
                }
                for opt in result.optional_dependencies {
                    if ctx.should_install_optional(&opt.lock_key, &opt.spec) {
                        let lock_key = opt.lock_key.clone();
                        ctx.enqueue(opt.spec, Some(opt.descriptor.clone()), true, Some(lock_key));
                    }
                }
                for peer in result.optional_peers {
                    if let Some(lock) = ctx.bun_lock.as_deref() {
                        if lock.get(&peer.lock_key).is_some() {
                            eprintln!(
                                "[DEBUG] optionalPeer {} exists in lockfile, installing",
                                peer.lock_key
                            );
                            let lock_key = peer.lock_key.clone();
                            ctx.enqueue(
                                peer.spec,
                                Some(peer.descriptor.clone()),
                                true,
                                Some(lock_key),
                            );
                        } else {
                            eprintln!(
                                "[DEBUG] optionalPeer {} not in lockfile, skipping",
                                peer.lock_key
                            );
                        }
                    }
                }

                ctx.completed_count += 1;
                emit_progress(
                    ctx.scheduled_count(),
                    ctx.completed_count,
                    ctx.pending_count(),
                )?;
                emit_package(
                    &spec_str,
                    "installed",
                    ecosystem.as_str(),
                    sequence,
                    ctx.scheduled_count(),
                    result.duration_ms,
                )?;
            }
            Err(err) => {
                if task.optional {
                    if !quiet {
                        eprintln!("helper    skipping optional {}: {}", spec_str, err);
                    }
                    ctx.completed_count += 1;
                    emit_progress(
                        ctx.scheduled_count(),
                        ctx.completed_count,
                        ctx.pending_count(),
                    )?;
                    continue;
                }
                return Err(err);
            }
        }
    }

    let duration = Instant::now().duration_since(start);
    emit_summary(ctx.installed.len(), duration.as_millis() as u64)?;
    Ok(())
}

pub fn run_probe(path: &PathBuf) -> Result<()> {
    let canonical = fs::canonicalize(path).context("failed to resolve probe path")?;
    emit_probe(&canonical)?;
    Ok(())
}

fn install_node_package(
    ctx: &mut InstallContext,
    spec: &str,
    lock_key: Option<&str>,
) -> Result<InstallResult> {
    let job_start = Instant::now();
    let (name, version_spec) = parse_package_spec(spec);
    let metadata = fetch_npm_metadata(&name)?;
    let lock_entry = ctx
        .bun_lock
        .as_deref()
        .and_then(|lock| lock.lookup(lock_key, &name));
    let version = lock_entry
        .map(|entry| entry.version.clone())
        .or_else(|| resolve_package_version(&metadata, version_spec.as_deref()))
        .context("unable to resolve package version")?;

    let version_info = metadata
        .get("versions")
        .and_then(|versions| versions.get(&version))
        .context("missing version info")?;
    let cache_key = cache_key(&name, &version);
    let cache_dir = ctx.cache.cache_dir(&cache_key);

    let (integrity, resolved) = if cache_dir.exists() {
        let meta_path = ctx.cache.metadata_path(&cache_key);
        let meta = crate::cache::read_metadata(&meta_path);
        let integrity = meta
            .as_ref()
            .and_then(|value| value.get("integrity"))
            .and_then(|value| value.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                version_info
                    .get("dist")
                    .and_then(|dist| dist.get("integrity"))
                    .and_then(|value| value.as_str())
                    .map(|s| s.to_string())
            })
            .context("integrity unavailable")?;
        let resolved = meta
            .as_ref()
            .and_then(|value| value.get("resolved"))
            .and_then(|value| value.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                version_info
                    .get("dist")
                    .and_then(|dist| dist.get("tarball"))
                    .and_then(|value| value.as_str())
                    .map(|s| s.to_string())
            })
            .context("resolved URL missing")?;
        (integrity, resolved)
    } else {
        let tarball_url = version_info
            .get("dist")
            .and_then(|dist| dist.get("tarball"))
            .and_then(|value| value.as_str())
            .context("tarball URL missing")?;
        let bytes = download_tarball(tarball_url)?;
        let integrity = version_info
            .get("dist")
            .and_then(|dist| dist.get("integrity"))
            .and_then(|value| value.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| compute_sha512(&bytes));
        let archive_path = ctx.cache.archive_path(&cache_key);
        fs::write(&archive_path, &bytes)?;
        extract_tarball(&archive_path, &cache_dir, &ctx.cache.tmp)?;
        let meta = json!({
            "integrity": integrity,
            "resolved": tarball_url,
            "version": version,
        });
        crate::cache::write_metadata(&ctx.cache.metadata_path(&cache_key), &meta)?;
        (integrity, tarball_url.to_string())
    };

    let install_path = ctx.determine_install_path(&name, &version, lock_key);

    if let Some(destination) = install_path {
        copy_package(&cache_dir, &destination)?;

        let descriptor = format!("{name}@{version}");
        let metadata_value = build_lock_metadata(version_info);
        lock::update_lock_entry(
            "node",
            &name,
            descriptor,
            resolved,
            metadata_value,
            integrity.clone(),
        )?;

        let path_str = destination.to_string_lossy();
        let install_key = format!("{}:{}@{}", path_str, name, version);
        ctx.installed.insert(install_key);
    }

    let dependencies = build_dependency_specs(
        lock_key,
        collect_spec_list(version_info.get("dependencies")),
        ctx.bun_lock.as_deref(),
    );
    let optional_dependencies = build_dependency_specs(
        lock_key,
        collect_spec_list(version_info.get("optionalDependencies")),
        ctx.bun_lock.as_deref(),
    );

    let optional_peers = if let Some(entry) = lock_entry {
        let peer_names = collect_optional_peers(Some(&entry.metadata));
        let peer_deps = version_info.get("peerDependencies");
        let peer_specs: Vec<String> = peer_names
            .into_iter()
            .filter_map(|name| {
                if let Some(Value::Object(peers)) = peer_deps {
                    peers.get(&name).and_then(|v| v.as_str()).map(|range| {
                        eprintln!("[DEBUG] optionalPeer found: {}@{}", name, range);
                        format!("{name}@{range}")
                    })
                } else {
                    None
                }
            })
            .collect();
        build_dependency_specs(lock_key, peer_specs, ctx.bun_lock.as_deref())
    } else {
        Vec::new()
    };

    let duration_ms = job_start.elapsed().as_millis() as u64;
    Ok(InstallResult {
        dependencies,
        optional_dependencies,
        optional_peers,
        duration_ms,
    })
}

fn collect_spec_list(value: Option<&Value>) -> Vec<String> {
    if let Some(Value::Object(map)) = value {
        map.iter()
            .filter_map(|(name, version)| version.as_str().map(|range| format!("{name}@{range}")))
            .collect()
    } else {
        Vec::new()
    }
}

fn collect_optional_peers(lock_entry: Option<&Value>) -> Vec<String> {
    if let Some(entry) = lock_entry {
        if let Some(Value::Array(peers)) = entry.get("optionalPeers") {
            return peers
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
        }
    }
    Vec::new()
}

fn build_lock_metadata(value: &Value) -> Value {
    let mut map = Map::new();
    for key in [
        "dependencies",
        "peerDependencies",
        "optionalDependencies",
        "bin",
    ] {
        if let Some(val) = value.get(key) {
            if !val.is_null() {
                map.insert(key.to_string(), val.clone());
            }
        }
    }
    Value::Object(map)
}

fn collect_project_specs() -> Result<Vec<String>> {
    let manifest = fs::read_to_string("package.json")
        .context("package.json not found in current directory")?;
    let pkg_json: Value =
        serde_json::from_str(&manifest).context("failed to parse package.json")?;
    let mut deps = BTreeMap::new();
    for section in ["dependencies", "devDependencies"] {
        if let Some(Value::Object(map)) = pkg_json.get(section) {
            for (name, value) in map {
                if let Some(version) = value.as_str() {
                    deps.insert(name.clone(), version.to_string());
                }
            }
        }
    }

    let lock = lock::read_lockfile();
    let mut specs = Vec::new();
    for (name, version) in deps {
        if let Some(entry) = lock.node.packages.get(&name) {
            specs.push(entry.0.clone());
        } else if !version.trim().is_empty() {
            specs.push(format!("{name}@{version}"));
        } else {
            specs.push(name.clone());
        }
    }
    Ok(specs)
}

fn prompt_yes_no(message: &str) -> Result<bool> {
    print!("  {} ", message);
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let normalized = input.trim().to_lowercase();
    Ok(normalized == "y" || normalized == "yes")
}

fn emit_event(value: &Value) -> Result<()> {
    println!("{}", value);
    Ok(())
}

fn emit_progress(scheduled: usize, completed: usize, active: usize) -> Result<()> {
    emit_event(&json!({
        "type": "progress",
        "scheduled": scheduled,
        "completed": completed,
        "active": active,
    }))
}

fn emit_package(
    name: &str,
    status: &str,
    ecosystem: &str,
    sequence: usize,
    total: usize,
    time_ms: u64,
) -> Result<()> {
    emit_event(&json!({
        "type": "package",
        "name": name,
        "status": status,
        "ecosystem": ecosystem,
        "sequence": sequence,
        "total": total,
        "timeMs": time_ms,
    }))
}

fn emit_summary(installed: usize, duration_ms: u64) -> Result<()> {
    emit_event(&json!({
        "type": "summary",
        "packagesInstalled": installed,
        "durationMs": duration_ms,
    }))
}

fn emit_probe(path: &PathBuf) -> Result<()> {
    emit_event(&json!({
        "type": "probe",
        "path": path,
        "status": "ok",
    }))
}

struct InstallContext {
    cache: CachePaths,
    queue: VecDeque<InstallTask>,
    scheduled: HashSet<String>,
    installed: HashSet<String>,
    bun_lock: Option<Arc<BunLock>>,
    completed_count: usize,
    current_os: String,
    current_cpu: String,
}

struct InstallTask {
    spec: String,
    optional: bool,
    lock_key: Option<String>,
}

impl InstallContext {
    fn new(cache: CachePaths, bun_lock: Option<Arc<BunLock>>) -> Self {
        Self {
            cache,
            queue: VecDeque::new(),
            scheduled: HashSet::new(),
            installed: HashSet::new(),
            bun_lock,
            completed_count: 0,
            current_os: normalize_os(),
            current_cpu: normalize_cpu(),
        }
    }

    fn enqueue(
        &mut self,
        spec: String,
        descriptor: Option<String>,
        optional: bool,
        lock_key: Option<String>,
    ) {
        let key = descriptor
            .as_deref()
            .map(|desc| desc.to_string())
            .unwrap_or_else(|| build_task_key(&spec, lock_key.as_deref()));
        if self.scheduled.insert(key) {
            self.queue.push_back(InstallTask {
                spec,
                optional,
                lock_key,
            });
        }
    }

    fn next_task(&mut self) -> Option<InstallTask> {
        self.queue.pop_front()
    }

    fn scheduled_count(&self) -> usize {
        self.scheduled.len()
    }

    fn pending_count(&self) -> usize {
        self.queue.len()
    }

    fn should_install_optional(&self, lock_key: &str, spec: &str) -> bool {
        if let Some(lock) = self.bun_lock.as_deref() {
            let (name, _) = parse_package_spec(spec);
            if let Some(entry) = lock.lookup(Some(lock_key), &name) {
                if !matches_requirement(entry.metadata.get("os"), &self.current_os) {
                    return false;
                }
                if !matches_requirement(entry.metadata.get("cpu"), &self.current_cpu) {
                    return false;
                }
            }
        }
        true
    }

    fn determine_install_path(
        &self,
        name: &str,
        version: &str,
        lock_key: Option<&str>,
    ) -> Option<PathBuf> {
        let target_path = if let Some(key) = lock_key {
            let is_scoped = key.starts_with('@');
            let segments: Vec<&str> = key.split('/').collect();
            let is_nested = if is_scoped {
                segments.len() > 2
            } else {
                segments.len() > 1
            };

            if is_nested {
                let mut path = self.cache.node_modules.clone();
                let parent_segments = if is_scoped {
                    &segments[..segments.len() - 1]
                } else {
                    &segments[..segments.len() - 1]
                };

                for segment in parent_segments {
                    path = path.join(segment);
                }
                path = path.join("node_modules");

                let name_segments: Vec<&str> = name.split('/').collect();
                for segment in name_segments {
                    path = path.join(segment);
                }
                path
            } else {
                self.cache.project_path_for(name)
            }
        } else {
            self.cache.project_path_for(name)
        };

        let path_str = target_path.to_string_lossy();
        let install_key = format!("{}:{}@{}", path_str, name, version);

        if self.installed.contains(&install_key) {
            eprintln!(
                "[DEBUG] {} already installed at {:?}, skipping",
                install_key, target_path
            );
            return None;
        }

        eprintln!(
            "[DEBUG] {} (lock_key={:?}) -> {:?}",
            name, lock_key, target_path
        );
        Some(target_path)
    }
}

struct DependencySpec {
    spec: String,
    lock_key: String,
    descriptor: String,
}

struct InstallResult {
    dependencies: Vec<DependencySpec>,
    optional_dependencies: Vec<DependencySpec>,
    optional_peers: Vec<DependencySpec>,
    duration_ms: u64,
}

fn normalize_os() -> String {
    match std::env::consts::OS {
        "macos" => "darwin".to_string(),
        "windows" => "win32".to_string(),
        other => other.to_string(),
    }
}

fn normalize_cpu() -> String {
    match std::env::consts::ARCH {
        "aarch64" => "arm64".to_string(),
        "x86_64" => "x64".to_string(),
        "x86" | "i586" => "ia32".to_string(),
        other => other.to_string(),
    }
}

fn matches_requirement(value: Option<&Value>, current: &str) -> bool {
    match value {
        Some(Value::String(expected)) => expected == "none" || expected == current,
        Some(Value::Array(list)) => list
            .iter()
            .any(|item| matches_requirement(Some(item), current)),
        _ => true,
    }
}

fn build_task_key(spec: &str, lock_key: Option<&str>) -> String {
    lock_key
        .map(|key| key.to_string())
        .unwrap_or_else(|| spec.to_string())
}

fn build_child_lock_key(parent: Option<&str>, name: &str) -> String {
    if let Some(parent_key) = parent {
        format!("{parent_key}/{name}")
    } else {
        name.to_string()
    }
}

fn build_dependency_specs(
    parent_lock_key: Option<&str>,
    specs: Vec<String>,
    bun_lock: Option<&BunLock>,
) -> Vec<DependencySpec> {
    specs
        .into_iter()
        .map(|spec| {
            let (name, _) = parse_package_spec(&spec);
            let (lock_key, descriptor) = if let Some(lock) = bun_lock {
                if let Some(parent) = parent_lock_key {
                    let nested_key = build_child_lock_key(Some(parent), &name);
                    if let Some(entry) = lock.get(&nested_key) {
                        eprintln!("[DEBUG] {}: nested key '{}' found", name, nested_key);
                        (nested_key, entry.descriptor.clone())
                    } else if let Some(entry) = lock.get(&name) {
                        eprintln!(
                            "[DEBUG] {}: using top-level key '{}' (parent={})",
                            name, name, parent
                        );
                        (name.clone(), entry.descriptor.clone())
                    } else {
                        eprintln!("[DEBUG] {}: not in lockfile", name);
                        (name.clone(), spec.clone())
                    }
                } else {
                    if let Some(entry) = lock.get(&name) {
                        (name.clone(), entry.descriptor.clone())
                    } else {
                        (name.clone(), spec.clone())
                    }
                }
            } else {
                (name.clone(), spec.clone())
            };

            DependencySpec {
                spec,
                lock_key,
                descriptor,
            }
        })
        .collect()
}
