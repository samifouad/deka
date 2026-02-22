use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::path::{Path, PathBuf};
use std::pin::Pin;

use deno_core::ModuleLoadOptions;
use deno_core::ModuleLoadReferrer;
use deno_core::ModuleLoadResponse;
use deno_core::ModuleLoader;
use deno_core::ModuleSource;
use deno_core::ModuleSourceCode;
use deno_core::ModuleSpecifier;
use deno_core::ModuleType;
use deno_core::ResolutionKind;
use deno_core::resolve_import;
use deno_error::JsErrorBox;

use phpx_js::build_stdlib_prelude;
use phpx_js::compile_phpx_source_to_js;
use phpx_js::parse_source_module_meta;
use phpx_js::SourceModuleMeta;
use runtime_core::module_spec::{is_bare_module_specifier, module_spec_aliases};

#[derive(Clone)]
pub struct PhpxEsmLoader {
    project_root: PathBuf,
    cache_dir: PathBuf,
    entry_specifier: ModuleSpecifier,
    wrapper_specifier: ModuleSpecifier,
    prelude_specifier: ModuleSpecifier,
    prelude_source: String,
    sources: Rc<RefCell<HashMap<String, ModuleSourceCode>>>,
}

impl PhpxEsmLoader {
    pub fn new(project_root: PathBuf, entry_path: PathBuf) -> Result<Self, JsErrorBox> {
        let cache_dir = project_root.join(".cache").join("phpx_js");
        std::fs::create_dir_all(&cache_dir)
            .map_err(|err| JsErrorBox::generic(format!("failed to create {}: {}", cache_dir.display(), err)))?;
        let entry_specifier = ModuleSpecifier::from_file_path(&entry_path)
            .map_err(|_| JsErrorBox::generic("invalid entry module path"))?;
        let wrapper_specifier = ModuleSpecifier::from_file_path(entry_wrapper_path(&project_root))
            .map_err(|_| JsErrorBox::generic("invalid entry wrapper path"))?;
        let prelude_specifier = ModuleSpecifier::from_file_path(entry_prelude_path(&project_root))
            .map_err(|_| JsErrorBox::generic("invalid prelude path"))?;
        let prelude_source = build_stdlib_prelude(&project_root).unwrap_or_else(|err| {
            format!(
                "if (!globalThis.panic) {{ globalThis.panic = (msg) => {{ throw new Error(String(msg)); }}; }}\n\
// stdlib prelude failed: {}\n",
                err.replace('\n', " ")
            )
        });
        Ok(Self {
            project_root,
            cache_dir,
            entry_specifier,
            wrapper_specifier,
            prelude_specifier,
            prelude_source,
            sources: Rc::new(RefCell::new(HashMap::new())),
        })
    }

    fn cache_path_for(&self, path: &Path) -> PathBuf {
        let rel = path.strip_prefix(&self.project_root).unwrap_or(path);
        let mut out = self.cache_dir.join(rel);
        out.set_extension("js");
        out
    }

    fn load_js_source(&self, path: &Path) -> Result<ModuleSourceCode, JsErrorBox> {
        let text = std::fs::read_to_string(path)
            .map_err(|err| JsErrorBox::from_err(err))?;
        Ok(ModuleSourceCode::String(text.into()))
    }

    fn load_phpx_source(&self, path: &Path) -> Result<ModuleSourceCode, JsErrorBox> {
        let input = path
            .to_str()
            .ok_or_else(|| JsErrorBox::generic(format!("invalid path: {}", path.display())))?;
        let source = std::fs::read_to_string(path)
            .map_err(|err| JsErrorBox::from_err(err))?;
        let meta = parse_source_module_meta(&source);
        ensure_project_layout(&self.project_root, &meta)
            .map_err(|err| JsErrorBox::generic(err))?;
        let js = compile_phpx_source_to_js(&source, input, meta)
            .map_err(|err| JsErrorBox::generic(err))?;

        let cache_path = self.cache_path_for(path);
        if let Some(parent) = cache_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&cache_path, &js);

        Ok(ModuleSourceCode::String(js.into()))
    }

    fn resolve_phpx_module_spec(&self, specifier: &str) -> Option<PathBuf> {
        resolve_phpx_module_spec(&self.project_root, specifier)
    }

    fn resolve_path(&self, specifier: &str, referrer: &str) -> Result<ModuleSpecifier, JsErrorBox> {
        if is_bare_specifier(specifier) {
            if let Some(path) = self.resolve_phpx_module_spec(specifier) {
                return ModuleSpecifier::from_file_path(path)
                    .map_err(|_| JsErrorBox::generic("invalid module path"));
            }
            return Err(JsErrorBox::generic(format!(
                "unable to resolve module '{}'; check php_modules",
                specifier
            )));
        }

        let resolved = resolve_import(specifier, referrer).map_err(JsErrorBox::from_err)?;
        if resolved.scheme() == "file" {
            return Ok(resolved);
        }
        Err(JsErrorBox::generic(format!("unsupported module scheme: {}", resolved)))
    }

    fn load_source(&self, specifier: &ModuleSpecifier) -> Result<ModuleSource, JsErrorBox> {
        if specifier == &self.prelude_specifier {
            return Ok(ModuleSource::new(
                ModuleType::JavaScript,
                ModuleSourceCode::String(self.prelude_source.clone().into()),
                specifier,
                None,
            ));
        }
        if specifier == &self.wrapper_specifier {
            let wrapper = self.wrapper_source();
            return Ok(ModuleSource::new(
                ModuleType::JavaScript,
                ModuleSourceCode::String(wrapper.into()),
                specifier,
                None,
            ));
        }
        let path = specifier
            .to_file_path()
            .map_err(|_| JsErrorBox::generic("Only file:// URLs are supported"))?;
        let ext = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
        let mut code = match ext {
            "phpx" => self.load_phpx_source(&path)?,
            _ => self.load_js_source(&path)?,
        };
        if specifier == &self.entry_specifier {
            code = append_entry_footer(code);
        }
        Ok(ModuleSource::new(ModuleType::JavaScript, code, specifier, None))
    }

    fn wrapper_source(&self) -> String {
        let entry = self.entry_specifier.to_string();
        let template = "import \"__PRELUDE__\";\n\
import * as __dekaMain from \"__ENTRY__\";\n\
const __candidate = typeof __dekaMain.default !== \"undefined\"\n\
  ? __dekaMain.default\n\
  : typeof __dekaMain.app !== \"undefined\"\n\
  ? __dekaMain.app\n\
  : typeof __dekaMain.handler !== \"undefined\"\n\
  ? __dekaMain.handler\n\
  : __dekaMain;\n\
if (__dekaMain && __dekaMain.phpxBuildMode === \"scaffold\" && typeof globalThis.__dekaRuntime === \"object\") {\n\
  const __fallbackKey = String(__dekaMain.phpxFile || \"unknown\") + \":\" + String(__dekaMain.phpxBuildReason || \"unknown\");\n\
  if (!globalThis.__dekaReportedPhpxFallback) globalThis.__dekaReportedPhpxFallback = new Set();\n\
  if (!globalThis.__dekaReportedPhpxFallback.has(__fallbackKey)) {\n\
    globalThis.__dekaReportedPhpxFallback.add(__fallbackKey);\n\
    console.error(\"[phpx-js] subset transpile fallback: \" + String(__dekaMain.phpxFile || \"unknown\") + \"\\n\" +\n\
      \"  reason: \" + String(__dekaMain.phpxBuildReason || \"unknown\") + \"\\n\" +\n\
      \"  behavior: running via runtime fallback path for this module\\n\" +\n\
      \"  hint: simplify unsupported syntax or check transpiler diagnostics for this file\");\n\
  }\n\
  const __mode = globalThis.__dekaExecMode || \"request\";\n\
  if (__mode === \"module\" && typeof __dekaMain.runPhpx === \"function\") {\n\
    await __dekaMain.runPhpx(globalThis.__dekaRuntime);\n\
  }\n\
  if (__mode !== \"module\" && typeof globalThis.__dekaPhp === \"object\" && typeof __dekaPhp.servePhp === \"function\") {\n\
    globalThis.app = __dekaPhp.servePhp(__dekaMain.phpxFile);\n\
  }\n\
}\n\
if (typeof globalThis.app === \"undefined\" && typeof __candidate !== \"undefined\") {\n\
  if (typeof __candidate === \"function\" && typeof globalThis.__dekaNodeExpressAdapter === \"function\" && (typeof __candidate.handle === \"function\" || typeof __candidate.listen === \"function\")) {\n\
    globalThis.app = globalThis.__dekaNodeExpressAdapter(__candidate);\n\
  } else if (__candidate && typeof __candidate === \"object\" && !__candidate.__dekaServer && (typeof __candidate.fetch === \"function\" || typeof __candidate.routes === \"object\")) {\n\
    globalThis.app = globalThis.__deka.serve(__candidate);\n\
  } else {\n\
    globalThis.app = __candidate;\n\
  }\n\
}\n";
        template
            .replace("__PRELUDE__", &self.prelude_specifier.to_string())
            .replace("__ENTRY__", &entry)
    }
}

impl ModuleLoader for PhpxEsmLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _kind: ResolutionKind,
    ) -> Result<ModuleSpecifier, JsErrorBox> {
        self.resolve_path(specifier, referrer)
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<&ModuleLoadReferrer>,
        _options: ModuleLoadOptions,
    ) -> ModuleLoadResponse {
        let key = module_specifier.to_string();
        if let Some(code) = self.sources.borrow_mut().remove(&key) {
            return ModuleLoadResponse::Sync(Ok(ModuleSource::new(
                ModuleType::JavaScript,
                code,
                module_specifier,
                None,
            )));
        }

        ModuleLoadResponse::Sync(self.load_source(module_specifier))
    }

    fn prepare_load(
        &self,
        module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<String>,
        _maybe_content: Option<String>,
        _options: ModuleLoadOptions,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), JsErrorBox>>>> {
        let loader = self.clone();
        let spec = module_specifier.clone();
        Box::pin(async move {
            let source = loader.load_source(&spec)?;
            loader
                .sources
                .borrow_mut()
                .insert(spec.to_string(), source.code);
            Ok(())
        })
    }
}

pub fn resolve_project_root(entry_path: &Path) -> Result<PathBuf, String> {
    let start = if entry_path.is_dir() {
        entry_path.to_path_buf()
    } else {
        entry_path.parent().unwrap_or(Path::new(".")).to_path_buf()
    };

    for dir in start.ancestors() {
        if dir.join("deka.json").is_file() {
            return Ok(dir.to_path_buf());
        }
    }

    Err(format!(
        "deka runtime requires a deka.json project root (searched from {})",
        entry_path.display()
    ))
}

pub fn entry_wrapper_path(project_root: &Path) -> PathBuf {
    project_root
        .join(".cache")
        .join("phpx_js")
        .join("__deka_entry.js")
}

pub fn entry_prelude_path(project_root: &Path) -> PathBuf {
    project_root
        .join(".cache")
        .join("phpx_js")
        .join("__deka_prelude.js")
}

pub fn hash_module_graph(entry_path: &Path) -> Result<u64, String> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let project_root = resolve_project_root(entry_path)?;
    let mut visited: HashSet<PathBuf> = HashSet::new();
    let mut stack: Vec<PathBuf> = vec![entry_path.to_path_buf()];
    let mut hasher = DefaultHasher::new();

    while let Some(path) = stack.pop() {
        if !visited.insert(path.clone()) {
            continue;
        }
        let source = std::fs::read_to_string(&path)
            .map_err(|err| format!("failed to read {}: {}", path.display(), err))?;
        source.hash(&mut hasher);

        let ext = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
        if ext == "phpx" {
            let meta = parse_source_module_meta(&source);
            for decl in meta.imports {
                if let Some(resolved) =
                    resolve_import_path(&project_root, &path, decl.from.trim())
                {
                    stack.push(resolved);
                }
            }
        }
    }

    Ok(hasher.finish())
}

pub fn ensure_project_layout(project_root: &Path, meta: &SourceModuleMeta) -> Result<(), String> {
    let lock_path = project_root.join("deka.lock");
    if !lock_path.is_file() {
        return Err(format!(
            "deka runtime requires deka.lock at project root: {}",
            lock_path.display()
        ));
    }

    let stdlib_imports = collect_stdlib_imports(meta);
    if stdlib_imports.is_empty() {
        return Ok(());
    }

    let modules_dir = project_root.join("php_modules");
    if !modules_dir.is_dir() {
        return Err(format!(
            "deka runtime requires php_modules/ at project root when using stdlib imports ({})",
            stdlib_imports.join(", ")
        ));
    }

    let mut missing = Vec::new();
    for spec in stdlib_imports {
        if resolve_module_file(&modules_dir, &spec).is_none() {
            missing.push(spec);
        }
    }

    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "missing stdlib modules under {}: {}",
            modules_dir.display(),
            missing.join(", ")
        ))
    }
}

fn collect_stdlib_imports(meta: &SourceModuleMeta) -> Vec<String> {
    let mut seen = HashSet::new();
    for decl in &meta.imports {
        let spec = decl.from.trim();
        if is_stdlib_module_spec(spec) {
            seen.insert(spec.to_string());
        }
    }
    seen.into_iter().collect()
}

fn is_stdlib_module_spec(spec: &str) -> bool {
    if !is_bare_specifier(spec) || spec.starts_with("@user/") {
        return false;
    }

    spec.starts_with("component/")
        || spec.starts_with("deka/")
        || spec.starts_with("encoding/")
        || spec.starts_with("db/")
        || spec.starts_with("@deka/")
        || matches!(
            spec,
            "json"
                | "postgres"
                | "mysql"
                | "sqlite"
                | "bytes"
                | "buffer"
                | "tcp"
                | "tls"
                | "fs"
                | "crypto"
                | "jwt"
                | "cookies"
                | "auth"
                | "db"
        )
}

fn resolve_module_file(modules_dir: &Path, spec: &str) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    for alias in module_spec_aliases(spec) {
        candidates.push(modules_dir.join(format!("{}.phpx", alias)));
        candidates.push(modules_dir.join(format!("{}.php", alias)));
        candidates.push(modules_dir.join(alias.as_str()).join("index.phpx"));
        candidates.push(modules_dir.join(alias.as_str()).join("index.php"));
        candidates.push(modules_dir.join(alias.as_str()).join("index.js"));
        if alias.ends_with(".phpx") || alias.ends_with(".php") || alias.ends_with(".js") {
            candidates.push(modules_dir.join(alias));
        }
    }

    candidates.into_iter().find(|path| path.is_file())
}

fn resolve_phpx_module_spec(project_root: &Path, specifier: &str) -> Option<PathBuf> {
    let modules_dir = project_root.join("php_modules");
    for alias in module_spec_aliases(specifier) {
        let base = if alias.starts_with("@user/") {
            modules_dir.join("@user").join(alias.trim_start_matches("@user/"))
        } else {
            modules_dir.join(alias)
        };
        if let Some(resolved) = resolve_with_candidates(&base) {
            return Some(resolved);
        }
    }
    None
}

fn resolve_import_path(
    project_root: &Path,
    referrer: &Path,
    specifier: &str,
) -> Option<PathBuf> {
    if is_bare_specifier(specifier) {
        return resolve_phpx_module_spec(project_root, specifier);
    }

    if specifier.starts_with("http://") || specifier.starts_with("https://") {
        return None;
    }

    let base = if specifier.starts_with('/') {
        PathBuf::from(specifier)
    } else {
        referrer
            .parent()
            .unwrap_or(Path::new("."))
            .join(specifier)
    };
    resolve_with_candidates(&base)
}

fn resolve_with_candidates(target: &Path) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if target.extension().is_none() {
        candidates.push(target.with_extension("phpx"));
        candidates.push(target.with_extension("js"));
        candidates.push(target.join("index.phpx"));
        candidates.push(target.join("index.js"));
    }
    candidates.push(target.to_path_buf());

    for candidate in candidates {
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn is_bare_specifier(spec: &str) -> bool {
    is_bare_module_specifier(spec)
}

fn append_entry_footer(code: ModuleSourceCode) -> ModuleSourceCode {
    const FOOTER: &str = "\nif (typeof globalThis.app === \"undefined\" && typeof app !== \"undefined\") {\n\
  const __candidate = app;\n\
  if (typeof __candidate === \"function\" && typeof globalThis.__dekaNodeExpressAdapter === \"function\" && (typeof __candidate.handle === \"function\" || typeof __candidate.listen === \"function\")) {\n\
    globalThis.app = globalThis.__dekaNodeExpressAdapter(__candidate);\n\
  } else if (__candidate && typeof __candidate === \"object\" && !__candidate.__dekaServer && (typeof __candidate.fetch === \"function\" || typeof __candidate.routes === \"object\")) {\n\
    globalThis.app = globalThis.__deka.serve(__candidate);\n\
  } else {\n\
    globalThis.app = __candidate;\n\
  }\n\
}\n";

    match code {
        ModuleSourceCode::String(source) => {
            let mut text = source.to_owned();
            text.push_str(FOOTER);
            ModuleSourceCode::String(text.into())
        }
        other => other,
    }
}
