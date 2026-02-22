use anyhow::{Context as AnyhowContext, Result, bail};
use core::{CommandSpec, Context, FlagSpec, ParamSpec, Registry};
use serde_json::json;
use std::io::{self, Write};
use std::process::Command;
use stdio;

use crate::cli::auth_store;

const COMMAND: CommandSpec = CommandSpec {
    name: "publish",
    category: "package",
    summary: "publish a PHPX package release to Linkhash",
    aliases: &[],
    subcommands: &[],
    handler: cmd,
};

pub fn register(registry: &mut Registry) {
    registry.add_command(COMMAND);
    registry.add_param(ParamSpec {
        name: "--name",
        description: "package name (@scope/name)",
    });
    registry.add_param(ParamSpec {
        name: "--version",
        description: "package version (deprecated alias; use --pkg-version)",
    });
    registry.add_param(ParamSpec {
        name: "--pkg-version",
        description: "package version",
    });
    registry.add_param(ParamSpec {
        name: "--repo",
        description: "source git repo name in linkhash",
    });
    registry.add_param(ParamSpec {
        name: "--git-ref",
        description: "git ref to publish (default: HEAD)",
    });
    registry.add_param(ParamSpec {
        name: "--token",
        description: "PAT token (fallback: auth profile or LINKHASH_TOKEN)",
    });
    registry.add_param(ParamSpec {
        name: "--registry-url",
        description: "registry base URL (default: auth profile, LINKHASH_REGISTRY_URL, or http://localhost:8508)",
    });
    registry.add_param(ParamSpec {
        name: "--description",
        description: "package description",
    });
    registry.add_flag(FlagSpec {
        name: "--yes",
        aliases: &["-y"],
        description: "auto-apply publish guard-rail fixes",
    });
}

pub fn cmd(context: &Context) {
    match build_request(context) {
        Ok(request) => {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            if let Err(err) = runtime.block_on(run_publish(request)) {
                stdio::error("publish", &err.to_string());
            }
        }
        Err(err) => stdio::error("publish", &err.to_string()),
    }
}

struct PublishRequest {
    endpoint: String,
    token: String,
    payload: serde_json::Value,
}

fn build_request(context: &Context) -> Result<PublishRequest> {
    let params = &context.args.params;
    let profile = auth_store::load().ok().flatten();

    let mut name = params.get("--name").cloned().context("missing --name")?;
    validate_scoped_package_name(&name)?;

    let mut version = params
        .get("--pkg-version")
        .or_else(|| params.get("--version"))
        .cloned()
        .context("missing --pkg-version")?;

    let mut repo = params.get("--repo").cloned().context("missing --repo")?;

    let mut git_ref = params
        .get("--git-ref")
        .cloned()
        .unwrap_or_else(|| "HEAD".to_string());

    let token = params
        .get("--token")
        .cloned()
        .or_else(|| profile.as_ref().map(|p| p.token.clone()))
        .or_else(|| std::env::var("LINKHASH_TOKEN").ok())
        .context("missing --token (or run `deka login`, or set LINKHASH_TOKEN)")?;

    let registry = params
        .get("--registry-url")
        .cloned()
        .or_else(|| profile.as_ref().map(|p| p.registry_url.clone()))
        .or_else(|| std::env::var("LINKHASH_REGISTRY_URL").ok())
        .unwrap_or_else(|| "http://localhost:8508".to_string());

    let description = params
        .get("--description")
        .cloned()
        .unwrap_or_else(|| "Published with deka publish".to_string());

    let auto_apply =
        context.args.flags.contains_key("--yes") || context.args.flags.contains_key("-y");
    let local_manifest = load_local_deka_manifest();
    let mut planned_fixes: Vec<String> = Vec::new();
    let mut tag_to_create: Option<String> = None;

    if let Some(manifest) = local_manifest.as_ref() {
        if manifest.name != name {
            planned_fixes.push(format!(
                "align --name from `{}` to deka.json name `{}`",
                name, manifest.name
            ));
            name = manifest.name.clone();
        }
        if manifest.version != version {
            planned_fixes.push(format!(
                "align --pkg-version from `{}` to deka.json version `{}`",
                version, manifest.version
            ));
            version = manifest.version.clone();
        }
    }

    let expected_repo = package_basename_from_scoped(&name)?;
    if repo != expected_repo {
        planned_fixes.push(format!(
            "align --repo from `{}` to package repo name `{}`",
            repo, expected_repo
        ));
        repo = expected_repo;
    }

    let expected_tag = format!("v{}", version);
    if !local_git_ref_exists(&format!("refs/tags/{}^{{commit}}", expected_tag)) {
        planned_fixes.push(format!(
            "create missing git tag `{}` for version {}",
            expected_tag, version
        ));
        tag_to_create = Some(expected_tag.clone());
    }
    if git_ref != expected_tag {
        planned_fixes.push(format!(
            "align --git-ref from `{}` to `{}`",
            git_ref, expected_tag
        ));
        git_ref = expected_tag.clone();
    }

    if !planned_fixes.is_empty() {
        stdio::warn_simple("publish guard rails detected fixable mismatches:");
        for fix in &planned_fixes {
            stdio::warn_simple(&format!("  - {}", fix));
        }

        let apply = if auto_apply {
            true
        } else {
            prompt_yes_no("Apply these fixes now? [Y/n]: ", true).unwrap_or(false)
        };

        if apply {
            if let Some(tag) = tag_to_create.as_ref() {
                create_local_git_tag(tag)?;
                stdio::log("publish", &format!("created local tag {}", tag));
            }
        } else {
            stdio::warn_simple(
                "continuing without auto-fixes; publish may be rejected by registry guard rails",
            );
        }
    }

    validate_scoped_package_name(&name)?;

    let endpoint = format!("{}/api/packages/publish", registry.trim_end_matches('/'));
    let mut payload = json!({
        "name": name,
        "version": version,
        "repo": repo,
        "git_ref": git_ref,
        "description": description,
    });
    if let Some(manifest) = local_manifest {
        payload["manifest"] = manifest.raw;
    }

    Ok(PublishRequest {
        endpoint,
        token,
        payload,
    })
}

async fn run_publish(request: PublishRequest) -> Result<()> {
    let requested_name = request
        .payload
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let requested_version = request
        .payload
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let client = reqwest::Client::new();
    let preflight_endpoint = request
        .endpoint
        .replace("/api/packages/publish", "/api/packages/preflight");
    let preflight_response = client
        .post(&preflight_endpoint)
        .bearer_auth(&request.token)
        .json(&request.payload)
        .send()
        .await
        .with_context(|| format!("failed to send preflight request to {}", preflight_endpoint))?;

    let preflight_status = preflight_response.status();
    let preflight_payload: serde_json::Value = preflight_response
        .json()
        .await
        .context("failed to parse preflight response json")?;
    if !preflight_status.is_success() {
        let err_msg = preflight_payload
            .get("error")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| preflight_payload.to_string());
        bail!(
            "publish preflight failed ({}): {}",
            preflight_status,
            err_msg
        );
    }

    if let Some(preflight) = preflight_payload.get("preflight") {
        let required = preflight
            .get("required_bump")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let minimum = preflight
            .get("minimum_allowed_version")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        stdio::log(
            "publish",
            &format!("preflight: required bump={}, minimum={}", required, minimum),
        );
        if let Some(reasons) = preflight.get("reasons").and_then(|v| v.as_array()) {
            for reason in reasons.iter().filter_map(|v| v.as_str()) {
                stdio::log("publish", &format!("reason: {}", reason));
            }
        }
        if let Some(caps) = preflight.get("capabilities") {
            if let Some(detected) = caps.get("detected").and_then(|v| v.as_array()) {
                for cap in detected.iter().filter_map(|v| v.as_str()) {
                    stdio::log("publish", &format!("capability detected: {}", cap));
                }
            }
            if let Some(missing) = caps.get("missing").and_then(|v| v.as_array()) {
                for cap in missing.iter().filter_map(|v| v.as_str()) {
                    stdio::warn_simple(&format!(
                        "missing capability declaration in manifest: {}",
                        cap
                    ));
                }
            }
        }
        if let Some(issues) = preflight.get("issues").and_then(|v| v.as_array()) {
            for issue in issues {
                let code = issue
                    .get("code")
                    .and_then(|v| v.as_str())
                    .unwrap_or("API_ISSUE");
                let symbol = issue
                    .get("symbol")
                    .and_then(|v| v.as_str())
                    .unwrap_or("<unknown>");
                let message = issue
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("api issue");
                let old_source = issue
                    .get("old_source")
                    .and_then(|v| v.as_str())
                    .unwrap_or("-");
                let new_source = issue
                    .get("new_source")
                    .and_then(|v| v.as_str())
                    .unwrap_or("-");
                stdio::warn_simple(&format!(
                    "{} {}: {} (old: {}, new: {})",
                    code, symbol, message, old_source, new_source
                ));
            }
        }
        let allowed = preflight
            .get("allowed")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        if !allowed {
            bail!(
                "publish blocked by semver gate: requested version is below required minimum {}",
                minimum
            );
        }
    }

    let response = client
        .post(&request.endpoint)
        .bearer_auth(request.token)
        .json(&request.payload)
        .send()
        .await
        .with_context(|| format!("failed to send publish request to {}", request.endpoint))?;

    let status = response.status();
    let payload: serde_json::Value = response
        .json()
        .await
        .context("failed to parse publish response json")?;

    if !status.is_success() {
        let err_msg = payload
            .get("error")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| payload.to_string());
        bail!("publish failed ({}): {}", status, err_msg);
    }

    let release = payload
        .get("release")
        .cloned()
        .unwrap_or_else(|| serde_json::Value::Null);

    let package = release
        .get("package_name")
        .and_then(|v| v.as_str())
        .unwrap_or("<unknown>");
    let version = release
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("<unknown>");

    if !requested_name.is_empty() && package != requested_name {
        bail!(
            "publish response mismatch: requested package `{}`, got `{}`",
            requested_name,
            package
        );
    }
    if !requested_version.is_empty() && version != requested_version {
        bail!(
            "publish response mismatch: requested version `{}`, got `{}`",
            requested_version,
            version
        );
    }

    stdio::log("publish", &format!("published: {}@{}", package, version));
    Ok(())
}

fn validate_scoped_package_name(name: &str) -> Result<()> {
    if !name.starts_with('@') {
        bail!("package name must be scoped as @scope/name");
    }

    let mut parts = name.split('/');
    let scope = parts.next().unwrap_or("");
    let pkg = parts.next().unwrap_or("");
    if parts.next().is_some() || scope.len() <= 1 || pkg.is_empty() {
        bail!("package name must be in @scope/name format");
    }

    if !scope[1..]
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        bail!("scope contains invalid characters");
    }

    if !pkg
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        bail!("package name contains invalid characters");
    }

    Ok(())
}

fn package_basename_from_scoped(name: &str) -> Result<String> {
    validate_scoped_package_name(name)?;
    let mut parts = name.split('/');
    let _scope = parts.next().unwrap_or_default();
    let pkg = parts.next().unwrap_or_default();
    Ok(pkg.to_string())
}

#[derive(Clone)]
struct LocalManifest {
    name: String,
    version: String,
    raw: serde_json::Value,
}

fn load_local_deka_manifest() -> Option<LocalManifest> {
    let cwd = std::env::current_dir().ok()?;
    let raw = std::fs::read_to_string(cwd.join("deka.json")).ok()?;
    let json = serde_json::from_str::<serde_json::Value>(&raw).ok()?;
    let name = json.get("name")?.as_str()?.trim().to_string();
    let version = json.get("version")?.as_str()?.trim().to_string();
    if name.is_empty() || version.is_empty() {
        return None;
    }
    Some(LocalManifest {
        name,
        version,
        raw: json,
    })
}

fn local_git_ref_exists(reference: &str) -> bool {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--verify")
        .arg(reference)
        .output();
    match output {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}

fn create_local_git_tag(tag: &str) -> Result<()> {
    let status = Command::new("git")
        .arg("tag")
        .arg(tag)
        .status()
        .with_context(|| format!("failed to run `git tag {}`", tag))?;
    if !status.success() {
        bail!(
            "failed to create tag `{}`. create it manually with `git tag {}`",
            tag,
            tag
        );
    }
    Ok(())
}

fn prompt_yes_no(prompt: &str, default_yes: bool) -> Option<bool> {
    print!("{}", prompt);
    let _ = io::stdout().flush();
    let mut buf = String::new();
    if io::stdin().read_line(&mut buf).is_err() {
        return None;
    }
    let trimmed = buf.trim().to_ascii_lowercase();
    if trimmed.is_empty() {
        return Some(default_yes);
    }
    if trimmed == "y" || trimmed == "yes" {
        return Some(true);
    }
    if trimmed == "n" || trimmed == "no" {
        return Some(false);
    }
    Some(default_yes)
}
