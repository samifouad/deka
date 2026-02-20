use core::Context;
use runtime_core::security_policy::{
    RuleList, SecurityCliOverrides, merge_policy_with_cli, parse_deka_security_policy,
    policy_to_json,
};

pub struct ResolvedSecurityPolicy {
    pub policy_json: String,
    pub prompt_enabled: bool,
    pub summary: String,
    pub warnings: Vec<String>,
}

pub fn resolve_security_policy(context: &Context) -> Result<ResolvedSecurityPolicy, String> {
    let deka_json_path = context.env.cwd.join("deka.json");
    let root = if deka_json_path.is_file() {
        let raw = std::fs::read_to_string(&deka_json_path)
            .map_err(|err| format!("failed to read {}: {}", deka_json_path.display(), err))?;
        serde_json::from_str::<serde_json::Value>(&raw)
            .map_err(|err| format!("invalid JSON in {}: {}", deka_json_path.display(), err))?
    } else {
        serde_json::json!({})
    };

    let parsed = parse_deka_security_policy(&root);
    if parsed.has_errors() {
        let mut lines = Vec::new();
        for diag in parsed.diagnostics {
            if matches!(
                diag.level,
                runtime_core::security_policy::PolicyDiagnosticLevel::Error
            ) {
                lines.push(format!("{} at {}: {}", diag.code, diag.path, diag.message));
            }
        }
        return Err(format!(
            "invalid security policy:\n{}",
            lines.join("\n")
        ));
    }

    let warnings = parsed
        .diagnostics
        .iter()
        .filter(|diag| {
            matches!(
                diag.level,
                runtime_core::security_policy::PolicyDiagnosticLevel::Warning
            )
        })
        .map(|diag| format!("{} at {}: {}", diag.code, diag.path, diag.message))
        .collect::<Vec<_>>();

    let overrides = SecurityCliOverrides::from_flags(&context.args.flags);
    let mut policy = parsed.policy;
    if context.args.flags.contains_key("--dev") {
        apply_dev_defaults(&mut policy, &context.handler.resolved.directory);
    }
    let merged = merge_policy_with_cli(policy, &overrides);
    let policy_json = serde_json::to_string(&policy_to_json(&merged))
        .map_err(|err| format!("failed to serialize security policy: {}", err))?;
    let summary = format!(
        "default-deny; allow(run={}, dynamic={}, wasm={}) deny(run={}, dynamic={}, net={}) prompt={}",
        summarize_rule(&merged.allow.run),
        merged.allow.dynamic,
        summarize_rule(&merged.allow.wasm),
        summarize_rule(&merged.deny.run),
        merged.deny.dynamic,
        summarize_rule(&merged.deny.net),
        merged.prompt
    );

    Ok(ResolvedSecurityPolicy {
        policy_json,
        prompt_enabled: merged.prompt,
        summary,
        warnings,
    })
}

fn apply_dev_defaults(policy: &mut runtime_core::security_policy::SecurityPolicy, root: &std::path::Path) {
    if matches!(policy.allow.read, RuleList::None) {
        policy.allow.read = RuleList::List(vec![root.to_string_lossy().to_string()]);
    }
    if matches!(policy.allow.write, RuleList::None) {
        let cache_dirs = vec![
            root.join(".cache"),
            root.join("php_modules").join(".cache"),
        ];
        let entries = cache_dirs
            .into_iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect();
        policy.allow.write = RuleList::List(entries);
    }
    if matches!(policy.allow.wasm, RuleList::None) {
        policy.allow.wasm = RuleList::All;
    }
    if matches!(policy.allow.env, RuleList::None) {
        policy.allow.env = RuleList::All;
    }
}

fn summarize_rule(rule: &RuleList) -> String {
    match rule {
        RuleList::None => "none".to_string(),
        RuleList::All => "all".to_string(),
        RuleList::List(items) => {
            if items.len() <= 3 {
                items.join(",")
            } else {
                format!("{} items", items.len())
            }
        }
    }
}
