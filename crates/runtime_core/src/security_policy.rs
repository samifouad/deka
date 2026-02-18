use serde_json::{Map, Value, json};
use std::collections::{BTreeSet, HashMap};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleList {
    None,
    All,
    List(Vec<String>),
}

impl RuleList {
    pub fn is_empty(&self) -> bool {
        matches!(self, RuleList::None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityScope {
    pub read: RuleList,
    pub write: RuleList,
    pub net: RuleList,
    pub env: RuleList,
    pub run: RuleList,
    pub db: RuleList,
    pub wasm: RuleList,
    pub dynamic: bool,
}

impl Default for SecurityScope {
    fn default() -> Self {
        Self {
            read: RuleList::None,
            write: RuleList::None,
            net: RuleList::None,
            env: RuleList::None,
            run: RuleList::None,
            db: RuleList::None,
            wasm: RuleList::None,
            dynamic: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityPolicy {
    pub allow: SecurityScope,
    pub deny: SecurityScope,
    pub prompt: bool,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            allow: SecurityScope::default(),
            deny: SecurityScope::default(),
            prompt: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyDiagnosticLevel {
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyDiagnostic {
    pub level: PolicyDiagnosticLevel,
    pub code: &'static str,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct PolicyParseOutcome {
    pub policy: SecurityPolicy,
    pub diagnostics: Vec<PolicyDiagnostic>,
}

impl PolicyParseOutcome {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diag| matches!(diag.level, PolicyDiagnosticLevel::Error))
    }
}

#[derive(Debug, Clone, Default)]
pub struct SecurityCliOverrides {
    pub allow_all: bool,
    pub allow_read: bool,
    pub allow_write: bool,
    pub allow_net: bool,
    pub allow_env: bool,
    pub allow_run: bool,
    pub allow_db: bool,
    pub allow_dynamic: bool,
    pub allow_wasm: bool,
    pub deny_read: bool,
    pub deny_write: bool,
    pub deny_net: bool,
    pub deny_env: bool,
    pub deny_run: bool,
    pub deny_db: bool,
    pub deny_dynamic: bool,
    pub deny_wasm: bool,
    pub no_prompt: bool,
}

impl SecurityCliOverrides {
    pub fn from_flags(flags: &HashMap<String, bool>) -> Self {
        Self {
            allow_all: flag_set(flags, "--allow-all"),
            allow_read: flag_set(flags, "--allow-read"),
            allow_write: flag_set(flags, "--allow-write"),
            allow_net: flag_set(flags, "--allow-net"),
            allow_env: flag_set(flags, "--allow-env"),
            allow_run: flag_set(flags, "--allow-run"),
            allow_db: flag_set(flags, "--allow-db"),
            allow_dynamic: flag_set(flags, "--allow-dynamic"),
            allow_wasm: flag_set(flags, "--allow-wasm"),
            deny_read: flag_set(flags, "--deny-read"),
            deny_write: flag_set(flags, "--deny-write"),
            deny_net: flag_set(flags, "--deny-net"),
            deny_env: flag_set(flags, "--deny-env"),
            deny_run: flag_set(flags, "--deny-run"),
            deny_db: flag_set(flags, "--deny-db"),
            deny_dynamic: flag_set(flags, "--deny-dynamic"),
            deny_wasm: flag_set(flags, "--deny-wasm"),
            no_prompt: flag_set(flags, "--no-prompt"),
        }
    }
}

pub fn merge_policy_with_cli(
    mut base: SecurityPolicy,
    cli: &SecurityCliOverrides,
) -> SecurityPolicy {
    if cli.allow_all {
        base.allow.read = RuleList::All;
        base.allow.write = RuleList::All;
        base.allow.net = RuleList::All;
        base.allow.env = RuleList::All;
        base.allow.run = RuleList::All;
        base.allow.db = RuleList::All;
        base.allow.wasm = RuleList::All;
        base.allow.dynamic = true;
    }

    if cli.allow_read {
        base.allow.read = RuleList::All;
    }
    if cli.allow_write {
        base.allow.write = RuleList::All;
    }
    if cli.allow_net {
        base.allow.net = RuleList::All;
    }
    if cli.allow_env {
        base.allow.env = RuleList::All;
    }
    if cli.allow_run {
        base.allow.run = RuleList::All;
    }
    if cli.allow_db {
        base.allow.db = RuleList::All;
    }
    if cli.allow_wasm {
        base.allow.wasm = RuleList::All;
    }
    if cli.allow_dynamic {
        base.allow.dynamic = true;
    }

    if cli.deny_read {
        base.deny.read = RuleList::All;
    }
    if cli.deny_write {
        base.deny.write = RuleList::All;
    }
    if cli.deny_net {
        base.deny.net = RuleList::All;
    }
    if cli.deny_env {
        base.deny.env = RuleList::All;
    }
    if cli.deny_run {
        base.deny.run = RuleList::All;
    }
    if cli.deny_db {
        base.deny.db = RuleList::All;
    }
    if cli.deny_wasm {
        base.deny.wasm = RuleList::All;
    }
    if cli.deny_dynamic {
        base.deny.dynamic = true;
    }

    if cli.no_prompt {
        base.prompt = false;
    }

    base
}

pub fn policy_to_json(policy: &SecurityPolicy) -> Value {
    json!({
        "deka.security": {
            "allow": scope_to_json(&policy.allow),
            "deny": scope_to_json(&policy.deny),
            "prompt": policy.prompt
        }
    })
}

pub fn parse_deka_security_policy(root: &Value) -> PolicyParseOutcome {
    let mut diagnostics = Vec::new();
    let mut policy = SecurityPolicy::default();
    let Some(obj) = root.as_object() else {
        diagnostics.push(diag(
            PolicyDiagnosticLevel::Error,
            "SECURITY_POLICY_ROOT_NOT_OBJECT",
            "$",
            "Expected JSON object at document root",
        ));
        return PolicyParseOutcome {
            policy,
            diagnostics,
        };
    };

    let Some(security) = obj.get("deka.security") else {
        return PolicyParseOutcome {
            policy,
            diagnostics,
        };
    };

    let Some(security_obj) = security.as_object() else {
        diagnostics.push(diag(
            PolicyDiagnosticLevel::Error,
            "SECURITY_POLICY_INVALID_TYPE",
            "$.deka.security",
            "Expected object for `deka.security`",
        ));
        return PolicyParseOutcome {
            policy,
            diagnostics,
        };
    };

    for key in security_obj.keys() {
        if key != "allow" && key != "deny" && key != "prompt" {
            diagnostics.push(diag(
                PolicyDiagnosticLevel::Warning,
                "SECURITY_POLICY_UNKNOWN_KEY",
                &format!("$.deka.security.{}", key),
                "Unknown key in `deka.security`",
            ));
        }
    }

    if let Some(allow) = security_obj.get("allow") {
        policy.allow = parse_scope("$.deka.security.allow", allow, &mut diagnostics);
    }
    if let Some(deny) = security_obj.get("deny") {
        policy.deny = parse_scope("$.deka.security.deny", deny, &mut diagnostics);
    }
    if let Some(prompt) = security_obj.get("prompt") {
        if let Some(value) = prompt.as_bool() {
            policy.prompt = value;
        } else {
            diagnostics.push(diag(
                PolicyDiagnosticLevel::Error,
                "SECURITY_POLICY_INVALID_PROMPT",
                "$.deka.security.prompt",
                "Expected boolean for `prompt`",
            ));
        }
    }

    PolicyParseOutcome {
        policy,
        diagnostics,
    }
}

fn parse_scope(
    path: &str,
    value: &Value,
    diagnostics: &mut Vec<PolicyDiagnostic>,
) -> SecurityScope {
    let mut scope = SecurityScope::default();
    let Some(obj) = value.as_object() else {
        diagnostics.push(diag(
            PolicyDiagnosticLevel::Error,
            "SECURITY_POLICY_SCOPE_NOT_OBJECT",
            path,
            "Expected object for security scope",
        ));
        return scope;
    };

    for key in obj.keys() {
        if key != "read"
            && key != "write"
            && key != "net"
            && key != "env"
            && key != "run"
            && key != "db"
            && key != "wasm"
            && key != "dynamic"
        {
            diagnostics.push(diag(
                PolicyDiagnosticLevel::Warning,
                "SECURITY_POLICY_UNKNOWN_SCOPE_KEY",
                &format!("{}.{}", path, key),
                "Unknown capability key in security scope",
            ));
        }
    }

    scope.read = parse_rule_list(&format!("{}.read", path), obj.get("read"), diagnostics);
    scope.write = parse_rule_list(&format!("{}.write", path), obj.get("write"), diagnostics);
    scope.net = parse_rule_list(&format!("{}.net", path), obj.get("net"), diagnostics);
    scope.env = parse_rule_list(&format!("{}.env", path), obj.get("env"), diagnostics);
    scope.run = parse_rule_list(&format!("{}.run", path), obj.get("run"), diagnostics);
    scope.db = parse_rule_list(&format!("{}.db", path), obj.get("db"), diagnostics);
    scope.wasm = parse_rule_list(&format!("{}.wasm", path), obj.get("wasm"), diagnostics);

    if let Some(dynamic) = obj.get("dynamic") {
        if let Some(v) = dynamic.as_bool() {
            scope.dynamic = v;
        } else {
            diagnostics.push(diag(
                PolicyDiagnosticLevel::Error,
                "SECURITY_POLICY_INVALID_DYNAMIC",
                &format!("{}.dynamic", path),
                "Expected boolean for `dynamic`",
            ));
        }
    }

    scope
}

fn parse_rule_list(
    path: &str,
    value: Option<&Value>,
    diagnostics: &mut Vec<PolicyDiagnostic>,
) -> RuleList {
    let Some(value) = value else {
        return RuleList::None;
    };

    if let Some(flag) = value.as_bool() {
        return if flag { RuleList::All } else { RuleList::None };
    }

    if let Some(single) = value.as_str() {
        let item = single.trim();
        if item.is_empty() {
            diagnostics.push(diag(
                PolicyDiagnosticLevel::Error,
                "SECURITY_POLICY_EMPTY_RULE_ITEM",
                path,
                "Empty rule item is not allowed",
            ));
            return RuleList::None;
        }
        return RuleList::List(vec![item.to_string()]);
    }

    if let Some(items) = value.as_array() {
        let mut set = BTreeSet::new();
        for (idx, item) in items.iter().enumerate() {
            let Some(raw) = item.as_str() else {
                diagnostics.push(diag(
                    PolicyDiagnosticLevel::Error,
                    "SECURITY_POLICY_RULE_ITEM_NOT_STRING",
                    &format!("{}[{}]", path, idx),
                    "Rule list entries must be strings",
                ));
                continue;
            };
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                diagnostics.push(diag(
                    PolicyDiagnosticLevel::Error,
                    "SECURITY_POLICY_EMPTY_RULE_ITEM",
                    &format!("{}[{}]", path, idx),
                    "Rule list entries must not be empty",
                ));
                continue;
            }
            set.insert(trimmed.to_string());
        }
        if set.is_empty() {
            return RuleList::None;
        }
        return RuleList::List(set.into_iter().collect());
    }

    diagnostics.push(diag(
        PolicyDiagnosticLevel::Error,
        "SECURITY_POLICY_INVALID_RULE_TYPE",
        path,
        "Expected boolean, string, or string array",
    ));
    RuleList::None
}

fn diag(
    level: PolicyDiagnosticLevel,
    code: &'static str,
    path: &str,
    message: &str,
) -> PolicyDiagnostic {
    PolicyDiagnostic {
        level,
        code,
        path: path.to_string(),
        message: message.to_string(),
    }
}

fn flag_set(flags: &HashMap<String, bool>, name: &str) -> bool {
    flags.get(name).copied().unwrap_or(false)
}

fn scope_to_json(scope: &SecurityScope) -> Value {
    let mut out = Map::new();
    out.insert("read".to_string(), rule_list_to_json(&scope.read));
    out.insert("write".to_string(), rule_list_to_json(&scope.write));
    out.insert("net".to_string(), rule_list_to_json(&scope.net));
    out.insert("env".to_string(), rule_list_to_json(&scope.env));
    out.insert("run".to_string(), rule_list_to_json(&scope.run));
    out.insert("db".to_string(), rule_list_to_json(&scope.db));
    out.insert("wasm".to_string(), rule_list_to_json(&scope.wasm));
    out.insert("dynamic".to_string(), Value::Bool(scope.dynamic));
    Value::Object(out)
}

fn rule_list_to_json(rule: &RuleList) -> Value {
    match rule {
        RuleList::None => Value::Bool(false),
        RuleList::All => Value::Bool(true),
        RuleList::List(items) => {
            Value::Array(items.iter().map(|v| Value::String(v.clone())).collect())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PolicyDiagnosticLevel, RuleList, SecurityCliOverrides, merge_policy_with_cli,
        parse_deka_security_policy, policy_to_json,
    };

    #[test]
    fn default_policy_when_key_missing() {
        let doc = serde_json::json!({ "name": "app" });
        let out = parse_deka_security_policy(&doc);
        assert!(!out.has_errors());
        assert!(matches!(out.policy.allow.read, RuleList::None));
        assert!(out.policy.prompt);
    }

    #[test]
    fn parses_allow_and_deny_scope() {
        let doc = serde_json::json!({
            "deka.security": {
                "allow": {
                    "read": ["./src", "./src", "./db"],
                    "run": "git",
                    "dynamic": false,
                    "wasm": true
                },
                "deny": {
                    "run": ["bash", "sh"],
                    "dynamic": true
                },
                "prompt": false
            }
        });
        let out = parse_deka_security_policy(&doc);
        assert!(!out.has_errors());
        assert_eq!(
            out.policy.allow.read,
            RuleList::List(vec!["./db".to_string(), "./src".to_string()])
        );
        assert_eq!(
            out.policy.allow.run,
            RuleList::List(vec!["git".to_string()])
        );
        assert_eq!(out.policy.allow.wasm, RuleList::All);
        assert_eq!(
            out.policy.deny.run,
            RuleList::List(vec!["bash".to_string(), "sh".to_string()])
        );
        assert!(out.policy.deny.dynamic);
        assert!(!out.policy.prompt);
    }

    #[test]
    fn emits_errors_for_invalid_shapes() {
        let doc = serde_json::json!({
            "deka.security": {
                "allow": {
                    "read": [true, ""],
                    "dynamic": "yes"
                },
                "prompt": "true"
            }
        });
        let out = parse_deka_security_policy(&doc);
        assert!(out.has_errors());
        assert!(
            out.diagnostics
                .iter()
                .any(|d| d.level == PolicyDiagnosticLevel::Error
                    && d.code == "SECURITY_POLICY_RULE_ITEM_NOT_STRING")
        );
        assert!(
            out.diagnostics
                .iter()
                .any(|d| d.level == PolicyDiagnosticLevel::Error
                    && d.code == "SECURITY_POLICY_INVALID_DYNAMIC")
        );
        assert!(
            out.diagnostics
                .iter()
                .any(|d| d.level == PolicyDiagnosticLevel::Error
                    && d.code == "SECURITY_POLICY_INVALID_PROMPT")
        );
    }

    #[test]
    fn merges_cli_flags_over_policy() {
        let doc = serde_json::json!({
            "deka.security": {
                "allow": { "read": ["./src"] },
                "deny": { "net": ["169.254.169.254"] },
                "prompt": true
            }
        });
        let parsed = parse_deka_security_policy(&doc);
        let merged = merge_policy_with_cli(
            parsed.policy,
            &SecurityCliOverrides {
                allow_net: true,
                deny_run: true,
                no_prompt: true,
                ..SecurityCliOverrides::default()
            },
        );
        assert_eq!(merged.allow.net, RuleList::All);
        assert_eq!(merged.deny.run, RuleList::All);
        assert!(!merged.prompt);
    }

    #[test]
    fn converts_policy_to_json_shape() {
        let parsed = parse_deka_security_policy(&serde_json::json!({
            "deka.security": {
                "allow": { "wasm": true, "dynamic": false },
                "deny": { "dynamic": true }
            }
        }));
        let out = policy_to_json(&parsed.policy);
        assert_eq!(
            out.pointer("/deka.security/allow/wasm")
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            out.pointer("/deka.security/deny/dynamic")
                .and_then(|v| v.as_bool()),
            Some(true)
        );
    }
}
