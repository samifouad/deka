use std::sync::Arc;

use crate::RuntimeState;
use crate::envelope::{RequestEnvelope, ResponseEnvelope};
use pool::{ExecutionMode, RequestData};

async fn execute_request_data(
    state: Arc<RuntimeState>,
    request_data: RequestData,
) -> Result<ResponseEnvelope, String> {
    let pool_response = state
        .engine
        .execute(state.handler_key.clone(), request_data)
        .await
        .map_err(|err| format!("handler execution failed: {}", err))?;

    if !pool_response.success {
        let error_msg = pool_response
            .error
            .unwrap_or_else(|| "Unknown error".to_string());
        return Err(format!("handler execution failed: {}", error_msg));
    }

    tracing::debug!(
        "Request completed - warm: {}µs, total: {}µs, cache_hit: {}",
        pool_response.warm_time_us,
        pool_response.total_time_us,
        pool_response.cache_hit
    );

    let result = pool_response
        .result
        .ok_or_else(|| "handler returned no result".to_string())?;

    ResponseEnvelope::from_value(result)
        .map_err(|err| format!("handler returned invalid response: {}", err))
}

pub async fn execute_request(
    state: Arc<RuntimeState>,
    request: RequestEnvelope,
) -> Result<ResponseEnvelope, String> {
    let request_value =
        serde_json::to_value(&request).map_err(|err| format!("serialize request: {}", err))?;

    let request_data = RequestData {
        handler_code: state.handler_code.clone(),
        request_value,
        mode: ExecutionMode::Request,
    };

    execute_request_data(state, request_data).await
}

pub async fn execute_request_value(
    state: Arc<RuntimeState>,
    request_value: serde_json::Value,
) -> Result<ResponseEnvelope, String> {
    let request_data = RequestData {
        handler_code: state.handler_code.clone(),
        request_value,
        mode: ExecutionMode::Request,
    };

    execute_request_data(state, request_data).await
}
