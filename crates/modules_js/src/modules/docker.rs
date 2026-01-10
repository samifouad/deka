//! deka/docker module
//!
//! Provides Docker container management
//!
//! TypeScript usage:
//! ```typescript
//! import { listContainers, createContainer } from 'deka/docker';
//!
//! const containers = await listContainers();
//! const container = await createContainer({
//!   image: 'nginx:latest',
//!   name: 'my-nginx',
//! });
//! ```

use deno_core::{error::CoreError, op2};

deno_core::extension!(
    deka_docker,
    ops = [
        op_docker_list_containers,
        op_docker_create_container,
    ],
    esm_entry_point = "ext:deka_docker/docker.js",
    esm = [ dir "src/modules", "docker.js" ],
);

/// Register all docker operations
pub fn register_ops() -> deno_core::Extension {
    deka_docker::init_ops_and_esm()
}

/// List all Docker containers
#[op2(async)]
#[serde]
async fn op_docker_list_containers() -> Result<Vec<serde_json::Value>, CoreError> {
    // TODO: Implement actual Docker API call
    // For now, return mock data
    Ok(vec![serde_json::json!({
        "id": "abc123",
        "name": "deka-agent-1",
        "status": "running"
    })])
}

/// Create a new Docker container
#[op2(async)]
#[serde]
async fn op_docker_create_container(
    #[serde] config: serde_json::Value,
) -> Result<serde_json::Value, CoreError> {
    // TODO: Implement actual Docker container creation
    // For now, return mock response
    Ok(serde_json::json!({
        "id": "new-container-123",
        "name": config.get("name").and_then(|v| v.as_str()).unwrap_or("unnamed")
    }))
}
