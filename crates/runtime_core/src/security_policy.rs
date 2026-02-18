use serde_json::Value;
use std::collections::BTreeSet;

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

#[cfg(test)]
mod tests {
    use super::{PolicyDiagnosticLevel, RuleList, parse_deka_security_policy};

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
}
