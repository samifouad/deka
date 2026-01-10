use redis::aio::ConnectionManager;
use redis::{Cmd, Value as RedisValue};
use std::collections::HashMap;
use std::sync::{
    OnceLock,
    atomic::{AtomicU64, Ordering},
};
use tokio::sync::Mutex;

struct RedisState {
    next_id: AtomicU64,
    clients: Mutex<HashMap<u64, ConnectionManager>>,
}

static REDIS_STATE: OnceLock<RedisState> = OnceLock::new();

fn state() -> &'static RedisState {
    REDIS_STATE.get_or_init(|| RedisState {
        next_id: AtomicU64::new(1),
        clients: Mutex::new(HashMap::new()),
    })
}

pub async fn connect(url: Option<String>) -> Result<u64, String> {
    let url = url.unwrap_or_else(default_redis_url);
    let client = redis::Client::open(url.as_str()).map_err(|err| err.to_string())?;
    let connection = client
        .get_connection_manager()
        .await
        .map_err(|err| err.to_string())?;

    let id = state().next_id.fetch_add(1, Ordering::Relaxed);
    let mut guard = state().clients.lock().await;
    guard.insert(id, connection);
    Ok(id)
}

pub async fn close(id: u64) -> Result<(), String> {
    let mut guard = state().clients.lock().await;
    guard.remove(&id);
    Ok(())
}

pub async fn execute(
    id: u64,
    command: &str,
    args: Vec<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    let mut guard = state().clients.lock().await;
    let connection = guard
        .get_mut(&id)
        .ok_or_else(|| format!("Redis client {} not found", id))?;

    let mut cmd = Cmd::new();
    cmd.arg(command);
    for arg in args {
        push_arg(&mut cmd, &arg);
    }

    let result: RedisValue = cmd
        .query_async(connection)
        .await
        .map_err(|err| err.to_string())?;

    Ok(redis_value_to_json(result))
}

pub async fn get_buffer(id: u64, key: String) -> Result<Vec<u8>, String> {
    let mut guard = state().clients.lock().await;
    let connection = guard
        .get_mut(&id)
        .ok_or_else(|| format!("Redis client {} not found", id))?;

    let mut cmd = Cmd::new();
    cmd.arg("GET").arg(key);

    let result: RedisValue = cmd
        .query_async(connection)
        .await
        .map_err(|err| err.to_string())?;

    match result {
        RedisValue::BulkString(data) => Ok(data),
        RedisValue::Nil => Ok(Vec::new()),
        other => Err(format!("Unexpected Redis response: {:?}", other)),
    }
}

fn push_arg(cmd: &mut Cmd, arg: &serde_json::Value) {
    match arg {
        serde_json::Value::Null => {
            cmd.arg("");
        }
        serde_json::Value::Bool(value) => {
            cmd.arg(if *value { "1" } else { "0" });
        }
        serde_json::Value::Number(value) => {
            cmd.arg(value.to_string());
        }
        serde_json::Value::String(value) => {
            cmd.arg(value);
        }
        serde_json::Value::Array(values) => {
            for value in values {
                push_arg(cmd, value);
            }
        }
        serde_json::Value::Object(_) => {
            cmd.arg(arg.to_string());
        }
    }
}

fn redis_value_to_json(value: RedisValue) -> serde_json::Value {
    match value {
        RedisValue::Nil => serde_json::Value::Null,
        RedisValue::Int(v) => serde_json::Value::Number(v.into()),
        RedisValue::BulkString(bytes) => {
            serde_json::Value::String(String::from_utf8_lossy(&bytes).to_string())
        }
        RedisValue::Okay => serde_json::Value::String("OK".to_string()),
        RedisValue::SimpleString(text) => serde_json::Value::String(text),
        RedisValue::Boolean(value) => serde_json::Value::Bool(value),
        RedisValue::Double(value) => serde_json::Number::from_f64(value)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        RedisValue::Array(values) => {
            serde_json::Value::Array(values.into_iter().map(redis_value_to_json).collect())
        }
        RedisValue::Map(values) => {
            let mut map = serde_json::Map::new();
            for (key, val) in values {
                let key = redis_value_to_json(key);
                let key = match key {
                    serde_json::Value::String(value) => value,
                    _ => key.to_string(),
                };
                map.insert(key, redis_value_to_json(val));
            }
            serde_json::Value::Object(map)
        }
        RedisValue::Attribute { data, attributes } => {
            let mut map = serde_json::Map::new();
            map.insert("data".to_string(), redis_value_to_json(*data));
            map.insert(
                "attributes".to_string(),
                serde_json::Value::Array(
                    attributes
                        .into_iter()
                        .map(|(key, value)| {
                            serde_json::Value::Array(vec![
                                redis_value_to_json(key),
                                redis_value_to_json(value),
                            ])
                        })
                        .collect(),
                ),
            );
            serde_json::Value::Object(map)
        }
        RedisValue::Set(values) => {
            serde_json::Value::Array(values.into_iter().map(redis_value_to_json).collect())
        }
        RedisValue::VerbatimString { text, .. } => serde_json::Value::String(text),
        RedisValue::BigNumber(value) => serde_json::Value::String(value.to_string()),
        RedisValue::Push { kind, data } => {
            let mut map = serde_json::Map::new();
            map.insert(
                "kind".to_string(),
                serde_json::Value::String(format!("{kind:?}")),
            );
            map.insert(
                "data".to_string(),
                serde_json::Value::Array(data.into_iter().map(redis_value_to_json).collect()),
            );
            serde_json::Value::Object(map)
        }
        RedisValue::ServerError(err) => {
            let mut map = serde_json::Map::new();
            map.insert(
                "code".to_string(),
                serde_json::Value::String(err.code().to_string()),
            );
            if let Some(detail) = err.details() {
                map.insert(
                    "detail".to_string(),
                    serde_json::Value::String(detail.to_string()),
                );
            }
            serde_json::Value::Object(map)
        }
    }
}

fn default_redis_url() -> String {
    std::env::var("REDIS_URL")
        .or_else(|_| std::env::var("VALKEY_URL"))
        .unwrap_or_else(|_| "redis://localhost:6379".to_string())
}
