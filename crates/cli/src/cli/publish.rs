use anyhow::{Context as AnyhowContext, Result, bail};
use core::{CommandSpec, Context, ParamSpec, Registry};
use serde_json::json;
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

    let name = params
        .get("--name")
        .map(String::as_str)
        .context("missing --name")?;
    validate_scoped_package_name(name)?;

    let version = params
        .get("--pkg-version")
        .or_else(|| params.get("--version"))
        .map(String::as_str)
        .context("missing --pkg-version")?;

    let repo = params
        .get("--repo")
        .map(String::as_str)
        .context("missing --repo")?;

    let git_ref = params
        .get("--git-ref")
        .map(String::as_str)
        .unwrap_or("HEAD");

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

    let endpoint = format!("{}/api/packages/publish", registry.trim_end_matches('/'));
    let payload = json!({
        "name": name,
        "version": version,
        "repo": repo,
        "git_ref": git_ref,
        "description": description,
    });

    Ok(PublishRequest {
        endpoint,
        token,
        payload,
    })
}

async fn run_publish(request: PublishRequest) -> Result<()> {
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
