use anyhow::{Context as AnyhowContext, Result, bail};
use base64::{Engine, engine::general_purpose::STANDARD};
use core::{CommandSpec, Context, ParamSpec, Registry};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};
use stdio;

const COMMAND: CommandSpec = CommandSpec {
    name: "publish",
    category: "package",
    summary: "publish a PHPX package artifact to a Linkhash-compatible registry",
    aliases: &[],
    subcommands: &[],
    handler: cmd,
};

pub fn register(registry: &mut Registry) {
    registry.add_command(COMMAND);
    registry.add_param(ParamSpec {
        name: "--org-id",
        description: "organization id in registry",
    });
    registry.add_param(ParamSpec {
        name: "--name",
        description: "package name",
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
        name: "--file",
        description: "artifact file path (tgz or single phpx file)",
    });
    registry.add_param(ParamSpec {
        name: "--token",
        description: "PAT token (fallback: LINKHASH_TOKEN env)",
    });
    registry.add_param(ParamSpec {
        name: "--registry-url",
        description: "registry base URL (default: LINKHASH_REGISTRY_URL or http://localhost:8508)",
    });
    registry.add_param(ParamSpec {
        name: "--visibility",
        description: "public|private (default: public)",
    });
    registry.add_param(ParamSpec {
        name: "--description",
        description: "package description",
    });
    registry.add_param(ParamSpec {
        name: "--lock-hash",
        description: "lock hash value (default: dev)",
    });
    registry.add_param(ParamSpec {
        name: "--main-file",
        description: "entrypoint path inside artifact (default: index.phpx)",
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
    form: Vec<(String, String)>,
}

fn build_request(context: &Context) -> Result<PublishRequest> {
    let params = &context.args.params;

    let org_id = params
        .get("--org-id")
        .map(String::as_str)
        .context("missing --org-id")?;
    let name = params
        .get("--name")
        .map(String::as_str)
        .context("missing --name")?;
    let version = params
        .get("--pkg-version")
        .or_else(|| params.get("--version"))
        .map(String::as_str)
        .context("missing --pkg-version")?;
    let file = params
        .get("--file")
        .map(String::as_str)
        .context("missing --file")?;

    let token = params
        .get("--token")
        .cloned()
        .or_else(|| std::env::var("LINKHASH_TOKEN").ok())
        .context("missing --token (or LINKHASH_TOKEN env)")?;

    let registry = params
        .get("--registry-url")
        .cloned()
        .or_else(|| std::env::var("LINKHASH_REGISTRY_URL").ok())
        .unwrap_or_else(|| "http://localhost:8508".to_string());

    let visibility = params
        .get("--visibility")
        .cloned()
        .unwrap_or_else(|| "public".to_string());
    if visibility != "public" && visibility != "private" {
        bail!("--visibility must be public or private");
    }

    let description = params
        .get("--description")
        .cloned()
        .unwrap_or_else(|| "Published with deka publish".to_string());

    let lock_hash = params
        .get("--lock-hash")
        .cloned()
        .unwrap_or_else(|| "dev".to_string());

    let main_file = params
        .get("--main-file")
        .cloned()
        .unwrap_or_else(|| "index.phpx".to_string());

    let artifact_path = PathBuf::from(file);
    let bytes = fs::read(&artifact_path)
        .with_context(|| format!("failed to read artifact file {}", artifact_path.display()))?;
    if bytes.is_empty() {
        bail!("artifact file is empty: {}", artifact_path.display());
    }

    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let sha256 = format!("{:x}", hasher.finalize());
    let artifact_b64 = STANDARD.encode(&bytes);

    let endpoint = format!("{}/api/publish", registry.trim_end_matches('/'));
    let form = vec![
        ("orgId".to_string(), org_id.to_string()),
        ("name".to_string(), name.to_string()),
        ("version".to_string(), version.to_string()),
        ("visibility".to_string(), visibility),
        ("description".to_string(), description),
        ("lockHash".to_string(), lock_hash),
        ("mainFile".to_string(), main_file),
        ("sha256".to_string(), sha256),
        (
            "mime".to_string(),
            "application/octet-stream".to_string(),
        ),
        ("artifactBase64".to_string(), artifact_b64),
    ];

    Ok(PublishRequest {
        endpoint,
        token,
        form,
    })
}

async fn run_publish(request: PublishRequest) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client
        .post(&request.endpoint)
        .bearer_auth(request.token)
        .form(&request.form)
        .send()
        .await
        .with_context(|| format!("failed to send publish request to {}", request.endpoint))?;

    let status = response.status();
    let payload: Value = response
        .json()
        .await
        .context("failed to parse publish response json")?;

    if !status.is_success() || !payload.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
        let err_msg = payload
            .get("error")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| payload.to_string());
        bail!(
            "publish failed ({}): {}",
            status,
            err_msg
        );
    }

    let canonical = payload
        .get("canonicalId")
        .and_then(|v| v.as_str())
        .unwrap_or("<unknown>");
    let url = payload
        .get("downloadUrl")
        .and_then(|v| v.as_str())
        .unwrap_or("<none>");

    stdio::log("publish", &format!("published: {}", canonical));
    stdio::log("publish", &format!("artifact: {}", url));
    Ok(())
}
