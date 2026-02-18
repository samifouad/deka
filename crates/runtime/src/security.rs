use core::Context;
use runtime_core::security_policy::{
    SecurityCliOverrides, merge_policy_with_cli, parse_deka_security_policy, policy_to_json,
};

pub struct ResolvedSecurityPolicy {
    pub policy_json: String,
    pub prompt_enabled: bool,
    pub enforce_enabled: bool,
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
            "invalid deka.security policy:\n{}",
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
    let has_cli_overrides = overrides.allow_all
        || overrides.allow_read
        || overrides.allow_write
        || overrides.allow_net
        || overrides.allow_env
        || overrides.allow_run
        || overrides.allow_db
        || overrides.allow_dynamic
        || overrides.allow_wasm
        || overrides.deny_read
        || overrides.deny_write
        || overrides.deny_net
        || overrides.deny_env
        || overrides.deny_run
        || overrides.deny_db
        || overrides.deny_dynamic
        || overrides.deny_wasm
        || overrides.no_prompt;
    let merged = merge_policy_with_cli(parsed.policy, &overrides);
    let policy_json = serde_json::to_string(&policy_to_json(&merged))
        .map_err(|err| format!("failed to serialize security policy: {}", err))?;

    Ok(ResolvedSecurityPolicy {
        policy_json,
        prompt_enabled: merged.prompt,
        enforce_enabled: root.get("deka.security").is_some() || has_cli_overrides,
        warnings,
    })
}
