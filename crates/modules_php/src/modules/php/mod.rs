// Minimal PHP runtime module - no heavy dependencies

use deno_core::op2;
use php_rs::parser::ast::{ClassKind, ClassMember, Program, Stmt, Type as AstType};
use php_rs::parser::lexer::Lexer;
use php_rs::parser::parser::{Parser, ParserMode};
use php_rs::parser::lexer::token::Token;
use bumpalo::Bump;
use mysql::prelude::Queryable;
use mysql::{OptsBuilder, Params as MyParams, Pool as MyPool, Value as MyValue};
use native_tls::{TlsConnector, TlsStream};
use postgres::{Client, NoTls, types::ToSql};
use prost::Message as ProstMessage;
use rusqlite::types::ValueRef as SqliteValueRef;
use rusqlite::{Connection as SqliteConnection, params_from_iter as sqlite_params_from_iter};
use std::collections::{HashMap, HashSet};
use std::fs::{File as StdFile, OpenOptions};
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use wit_parser::{Resolve, Results, Type, TypeDefKind, TypeId, WorldItem, WorldKey};

/// Embedded PHP WASM binary produced by the `php-rs` crate.
static PHP_WASM_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/php_rs.wasm"));

mod proto {
    pub mod bridge_v1 {
        include!(concat!(env!("OUT_DIR"), "/deka.bridge.v1.rs"));
    }
}

#[derive(serde::Serialize)]
struct PhpDirEntry {
    name: String,
    is_dir: bool,
    is_file: bool,
}

#[derive(serde::Serialize)]
struct WitSchema {
    world: String,
    functions: Vec<WitFunction>,
    interfaces: Vec<WitInterface>,
}

#[derive(serde::Serialize)]
struct WitInterface {
    name: String,
    functions: Vec<WitFunction>,
}

#[derive(serde::Serialize)]
struct WitFunction {
    name: String,
    params: Vec<WitParam>,
    result: Option<WitType>,
}

#[derive(serde::Serialize)]
struct WitParam {
    name: String,
    #[serde(rename = "type")]
    ty: WitType,
}

#[derive(serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum WitType {
    Bool,
    U8,
    U16,
    U32,
    U64,
    S8,
    S16,
    S32,
    S64,
    F32,
    F64,
    Char,
    String,
    List { element: Box<WitType> },
    Record { fields: Vec<WitField> },
    Tuple { items: Vec<WitType> },
    Option { some: Box<WitType> },
    Result { ok: Option<Box<WitType>>, err: Option<Box<WitType>> },
    Enum { cases: Vec<String> },
    Flags { flags: Vec<String> },
    Variant { cases: Vec<WitVariantCase> },
    Resource,
    Unsupported { detail: String },
}

#[derive(serde::Serialize)]
struct WitField {
    name: String,
    #[serde(rename = "type")]
    ty: WitType,
}

#[derive(serde::Serialize)]
struct WitVariantCase {
    name: String,
    #[serde(rename = "type")]
    ty: Option<WitType>,
}

#[derive(serde::Serialize, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum BridgeType {
    Unknown,
    Mixed,
    Primitive { name: String },
    Array { element: Option<Box<BridgeType>> },
    Object,
    ObjectShape { fields: Vec<BridgeField> },
    Struct { name: String, fields: Vec<BridgeField> },
    Enum { name: String },
    Union { types: Vec<BridgeType> },
    Option { inner: Option<Box<BridgeType>> },
    Result { ok: Option<Box<BridgeType>>, err: Option<Box<BridgeType>> },
    Applied { base: String, args: Vec<BridgeType> },
    TypeParam { name: String },
}

#[derive(serde::Serialize, Clone)]
struct BridgeField {
    name: String,
    #[serde(rename = "type")]
    ty: BridgeType,
    optional: bool,
}

#[derive(serde::Serialize)]
struct BridgeParam {
    #[serde(rename = "type")]
    ty: Option<BridgeType>,
    required: bool,
    variadic: bool,
}

#[derive(serde::Serialize)]
struct BridgeFunction {
    params: Vec<BridgeParam>,
    #[serde(rename = "return")]
    return_type: Option<BridgeType>,
    variadic: bool,
}

#[derive(serde::Serialize)]
struct BridgeStruct {
    fields: Vec<BridgeField>,
}

#[derive(serde::Serialize)]
struct BridgeModuleTypes {
    functions: HashMap<String, BridgeFunction>,
    structs: HashMap<String, BridgeStruct>,
}

struct DbConn {
    key: String,
    config: DbDriverConfig,
}

#[derive(Clone)]
struct PgConnConfig {
    host: String,
    port: u16,
    database: String,
    user: String,
    password: String,
}

#[derive(Clone)]
struct SqliteConnConfig {
    path: String,
}

#[derive(Clone)]
struct MysqlConnConfig {
    host: String,
    port: u16,
    database: String,
    user: String,
    password: String,
}

#[derive(Clone)]
enum DbDriverConfig {
    Postgres(PgConnConfig),
    Sqlite(SqliteConnConfig),
    Mysql(MysqlConnConfig),
}

impl DbDriverConfig {
    fn driver_name(&self) -> &'static str {
        match self {
            DbDriverConfig::Postgres(_) => "postgres",
            DbDriverConfig::Sqlite(_) => "sqlite",
            DbDriverConfig::Mysql(_) => "mysql",
        }
    }
}

#[derive(Clone, serde::Serialize)]
struct DbMetric {
    calls: u64,
    errors: u64,
    total_ms: u64,
}

struct DbState {
    next_handle: u64,
    handles: HashMap<u64, DbConn>,
    key_to_handle: HashMap<String, u64>,
    metrics: HashMap<String, DbMetric>,
}

impl DbState {
    fn new() -> Self {
        Self {
            next_handle: 1,
            handles: HashMap::new(),
            key_to_handle: HashMap::new(),
            metrics: HashMap::new(),
        }
    }

    fn record_metric(&mut self, action: &str, driver: &str, elapsed_ms: u64, is_error: bool) {
        let key = format!("{}:{}", action, driver);
        let metric = self.metrics.entry(key).or_insert(DbMetric {
            calls: 0,
            errors: 0,
            total_ms: 0,
        });
        metric.calls += 1;
        if is_error {
            metric.errors += 1;
        }
        metric.total_ms = metric.total_ms.saturating_add(elapsed_ms);
    }
}

static DB_STATE: OnceLock<Mutex<DbState>> = OnceLock::new();

fn db_state() -> &'static Mutex<DbState> {
    DB_STATE.get_or_init(|| Mutex::new(DbState::new()))
}

enum NetConn {
    Tcp(TcpStream),
    Tls(TlsStream<TcpStream>),
}

struct NetState {
    next_handle: u64,
    handles: HashMap<u64, NetConn>,
}

impl NetState {
    fn new() -> Self {
        Self {
            next_handle: 1,
            handles: HashMap::new(),
        }
    }
}

static NET_STATE: OnceLock<Mutex<NetState>> = OnceLock::new();

fn net_state() -> &'static Mutex<NetState> {
    NET_STATE.get_or_init(|| Mutex::new(NetState::new()))
}

struct FsState {
    next_handle: u64,
    handles: HashMap<u64, StdFile>,
}

impl FsState {
    fn new() -> Self {
        Self {
            next_handle: 1,
            handles: HashMap::new(),
        }
    }
}

static FS_STATE: OnceLock<Mutex<FsState>> = OnceLock::new();

fn fs_state() -> &'static Mutex<FsState> {
    FS_STATE.get_or_init(|| Mutex::new(FsState::new()))
}

fn sanitize_conn_value(value: &str) -> String {
    value
        .chars()
        .filter(|ch| !ch.is_control())
        .collect::<String>()
        .trim()
        .to_string()
}

fn with_pg_client<T>(
    cfg: PgConnConfig,
    f: impl FnOnce(&mut Client) -> Result<T, deno_core::error::CoreError> + Send + 'static,
) -> Result<T, deno_core::error::CoreError>
where
    T: Send + 'static,
{
    std::thread::spawn(move || {
        let host = sanitize_conn_value(&cfg.host);
        let user = sanitize_conn_value(&cfg.user);
        let database = sanitize_conn_value(&cfg.database);
        let password = sanitize_conn_value(&cfg.password);

        let mut dsn = format!(
            "host={} port={} user={} dbname={}",
            host, cfg.port, user, database
        );
        if !password.is_empty() {
            dsn.push_str(" password=");
            dsn.push_str(&password);
        }

        let url = if password.is_empty() {
            format!("postgres://{}@{}:{}/{}", user, host, cfg.port, database)
        } else {
            format!("postgres://{}:{}@{}:{}/{}", user, password, host, cfg.port, database)
        };

        let mut client = match Client::connect(&dsn, NoTls) {
            Ok(client) => client,
            Err(err_dsn) => Client::connect(&url, NoTls).map_err(|err_url| {
                deno_core::error::CoreError::from(std::io::Error::other(format!(
                    "postgres connect failed: {} (dsn={}); fallback failed: {} (url={})",
                    err_dsn, dsn, err_url, url
                )))
            })?,
        };

        f(&mut client)
    })
    .join()
    .map_err(|_| deno_core::error::CoreError::from(std::io::Error::other("db worker thread panicked")))?
}

fn json_to_pg_param(value: &serde_json::Value) -> Box<dyn ToSql + Sync> {
    match value {
        serde_json::Value::Null => Box::new(None::<String>),
        serde_json::Value::Bool(v) => Box::new(*v),
        serde_json::Value::Number(v) => {
            if let Some(i) = v.as_i64() {
                Box::new(i)
            } else if let Some(u) = v.as_u64() {
                if u <= i64::MAX as u64 {
                    Box::new(u as i64)
                } else {
                    Box::new(u as f64)
                }
            } else if let Some(f) = v.as_f64() {
                Box::new(f)
            } else {
                Box::new(v.to_string())
            }
        }
        serde_json::Value::String(v) => Box::new(v.clone()),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => Box::new(value.to_string()),
    }
}

fn pg_cell_to_json(row: &postgres::Row, idx: usize) -> serde_json::Value {
    let col = &row.columns()[idx];
    match col.type_().name() {
        "bool" => row
            .try_get::<usize, Option<bool>>(idx)
            .ok()
            .flatten()
            .map(serde_json::Value::Bool)
            .unwrap_or(serde_json::Value::Null),
        "int2" => row
            .try_get::<usize, Option<i16>>(idx)
            .ok()
            .flatten()
            .map(|v| serde_json::Value::Number(serde_json::Number::from(v as i64)))
            .unwrap_or(serde_json::Value::Null),
        "int4" => row
            .try_get::<usize, Option<i32>>(idx)
            .ok()
            .flatten()
            .map(|v| serde_json::Value::Number(serde_json::Number::from(v as i64)))
            .unwrap_or(serde_json::Value::Null),
        "int8" => row
            .try_get::<usize, Option<i64>>(idx)
            .ok()
            .flatten()
            .map(|v| serde_json::Value::Number(serde_json::Number::from(v)))
            .unwrap_or(serde_json::Value::Null),
        "float4" => row
            .try_get::<usize, Option<f32>>(idx)
            .ok()
            .flatten()
            .and_then(|v| serde_json::Number::from_f64(v as f64))
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        "float8" => row
            .try_get::<usize, Option<f64>>(idx)
            .ok()
            .flatten()
            .and_then(serde_json::Number::from_f64)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        "json" | "jsonb" => row
            .try_get::<usize, Option<String>>(idx)
            .ok()
            .flatten()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            .unwrap_or(serde_json::Value::Null),
        _ => row
            .try_get::<usize, Option<String>>(idx)
            .ok()
            .flatten()
            .map(serde_json::Value::String)
            .unwrap_or(serde_json::Value::Null),
    }
}

fn with_sqlite_conn<T>(
    cfg: SqliteConnConfig,
    f: impl FnOnce(&SqliteConnection) -> Result<T, deno_core::error::CoreError> + Send + 'static,
) -> Result<T, deno_core::error::CoreError>
where
    T: Send + 'static,
{
    std::thread::spawn(move || {
        let path = sanitize_conn_value(&cfg.path);
        let conn = SqliteConnection::open(&path).map_err(|e| {
            deno_core::error::CoreError::from(std::io::Error::other(format!(
                "sqlite open failed: {} (path={})",
                e, path
            )))
        })?;
        f(&conn)
    })
    .join()
    .map_err(|_| deno_core::error::CoreError::from(std::io::Error::other("db worker thread panicked")))?
}

fn json_to_sqlite_value(value: &serde_json::Value) -> rusqlite::types::Value {
    match value {
        serde_json::Value::Null => rusqlite::types::Value::Null,
        serde_json::Value::Bool(v) => rusqlite::types::Value::Integer(if *v { 1 } else { 0 }),
        serde_json::Value::Number(v) => {
            if let Some(i) = v.as_i64() {
                rusqlite::types::Value::Integer(i)
            } else if let Some(f) = v.as_f64() {
                rusqlite::types::Value::Real(f)
            } else {
                rusqlite::types::Value::Null
            }
        }
        serde_json::Value::String(v) => rusqlite::types::Value::Text(v.clone()),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            rusqlite::types::Value::Text(value.to_string())
        }
    }
}

fn sqlite_cell_to_json(row: &rusqlite::Row<'_>, idx: usize) -> serde_json::Value {
    match row.get_ref(idx) {
        Ok(SqliteValueRef::Null) => serde_json::Value::Null,
        Ok(SqliteValueRef::Integer(v)) => serde_json::Value::Number(serde_json::Number::from(v)),
        Ok(SqliteValueRef::Real(v)) => serde_json::Number::from_f64(v)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        Ok(SqliteValueRef::Text(v)) => {
            serde_json::Value::String(String::from_utf8_lossy(v).to_string())
        }
        Ok(SqliteValueRef::Blob(v)) => serde_json::Value::String(format!("{:?}", v)),
        Err(_) => serde_json::Value::Null,
    }
}

fn with_mysql_conn<T>(
    cfg: MysqlConnConfig,
    f: impl FnOnce(&mut mysql::PooledConn) -> Result<T, deno_core::error::CoreError> + Send + 'static,
) -> Result<T, deno_core::error::CoreError>
where
    T: Send + 'static,
{
    std::thread::spawn(move || {
        let host = sanitize_conn_value(&cfg.host);
        let user = sanitize_conn_value(&cfg.user);
        let database = sanitize_conn_value(&cfg.database);
        let password = sanitize_conn_value(&cfg.password);

        let opts = OptsBuilder::new()
            .ip_or_hostname(Some(host.clone()))
            .tcp_port(cfg.port)
            .user(Some(user.clone()))
            .pass(Some(password.clone()))
            .db_name(Some(database.clone()))
            .tcp_connect_timeout(Some(Duration::from_secs(3)))
            .read_timeout(Some(Duration::from_secs(5)))
            .write_timeout(Some(Duration::from_secs(5)));

        let pool = MyPool::new(opts).map_err(|e| {
            deno_core::error::CoreError::from(std::io::Error::other(format!(
                "mysql pool failed: {} (host={}, port={}, database={}, user={})",
                e, host, cfg.port, database, user
            )))
        })?;
        let mut conn = pool.get_conn().map_err(|e| {
            deno_core::error::CoreError::from(std::io::Error::other(format!(
                "mysql connect failed: {}",
                e
            )))
        })?;
        f(&mut conn)
    })
    .join()
    .map_err(|_| deno_core::error::CoreError::from(std::io::Error::other("db worker thread panicked")))?
}

fn json_to_mysql_value(value: &serde_json::Value) -> MyValue {
    match value {
        serde_json::Value::Null => MyValue::NULL,
        serde_json::Value::Bool(v) => MyValue::Int(if *v { 1 } else { 0 }),
        serde_json::Value::Number(v) => {
            if let Some(i) = v.as_i64() {
                MyValue::Int(i)
            } else if let Some(u) = v.as_u64() {
                MyValue::UInt(u)
            } else if let Some(f) = v.as_f64() {
                MyValue::Double(f)
            } else {
                MyValue::NULL
            }
        }
        serde_json::Value::String(v) => MyValue::Bytes(v.clone().into_bytes()),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            MyValue::Bytes(value.to_string().into_bytes())
        }
    }
}

fn mysql_value_to_json(value: &MyValue) -> serde_json::Value {
    match value {
        MyValue::NULL => serde_json::Value::Null,
        MyValue::Bytes(v) => serde_json::Value::String(String::from_utf8_lossy(v).to_string()),
        MyValue::Int(v) => serde_json::Value::Number(serde_json::Number::from(*v)),
        MyValue::UInt(v) => serde_json::Value::Number(serde_json::Number::from(*v)),
        MyValue::Float(v) => serde_json::Number::from_f64(*v as f64)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        MyValue::Double(v) => serde_json::Number::from_f64(*v)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        MyValue::Date(y, m, d, h, i, s, micros) => serde_json::Value::String(format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:06}",
            y, m, d, h, i, s, micros
        )),
        MyValue::Time(neg, days, h, i, s, micros) => serde_json::Value::String(format!(
            "{}{} {:02}:{:02}:{:02}.{:06}",
            if *neg { "-" } else { "" },
            days,
            h,
            i,
            s,
            micros
        )),
    }
}

#[derive(Clone)]
struct TypeAliasInfo<'a> {
    params: Vec<String>,
    ty: &'a AstType<'a>,
}

struct TypeResolver<'a> {
    source: &'a [u8],
    aliases: HashMap<String, TypeAliasInfo<'a>>,
    structs: HashMap<String, Vec<BridgeField>>,
}

impl<'a> TypeResolver<'a> {
    fn new(source: &'a [u8], program: &'a Program<'a>) -> Self {
        let aliases = collect_aliases(program, source);
        let structs = collect_structs(program, source, &aliases);
        Self {
            source,
            aliases,
            structs,
        }
    }

    fn type_name(&self, ty: &'a AstType<'a>) -> Option<String> {
        match ty {
            AstType::Simple(token) => Some(token_text(self.source, token)),
            AstType::Name(name) => Some(name_to_string(self.source, name)),
            _ => None,
        }
    }

    fn base_name(name: &str) -> &str {
        name.rsplit('\\').next().unwrap_or(name)
    }

    fn resolve_alias(&mut self, name: &str) -> Option<BridgeType> {
        let Some(alias) = self.aliases.get(name) else {
            return None;
        };
        if !alias.params.is_empty() {
            return Some(BridgeType::Mixed);
        }
        let mut guard = HashSet::new();
        Some(self.convert_type_with_guard(alias.ty, &mut guard))
    }

    fn convert_type(&mut self, ty: &'a AstType<'a>) -> BridgeType {
        let mut guard = HashSet::new();
        self.convert_type_internal(ty, &mut guard, None)
    }

    fn convert_type_with_guard(
        &mut self,
        ty: &'a AstType<'a>,
        alias_guard: &mut HashSet<String>,
    ) -> BridgeType {
        self.convert_type_internal(ty, alias_guard, None)
    }

    fn convert_type_internal(
        &mut self,
        ty: &'a AstType<'a>,
        alias_guard: &mut HashSet<String>,
        subs: Option<&HashMap<String, BridgeType>>,
    ) -> BridgeType {
        let resolve_param = |name: &str, subs: Option<&HashMap<String, BridgeType>>| {
            subs.and_then(|map| map.get(name).cloned())
        };
        match ty {
            AstType::Simple(token) => {
                let name = token_text(self.source, token);
                if let Some(bound) = resolve_param(&name, subs) {
                    return bound;
                }
                if let Some(resolved) = self.convert_named(&name, alias_guard) {
                    return resolved;
                }
                BridgeType::Unknown
            }
            AstType::Name(name) => {
                let name_str = name_to_string(self.source, name);
                if let Some(bound) = resolve_param(&name_str, subs) {
                    return bound;
                }
                if let Some(resolved) = self.convert_named(&name_str, alias_guard) {
                    return resolved;
                }
                BridgeType::Unknown
            }
            AstType::Nullable(inner) => {
                let inner = self.convert_type_internal(inner, alias_guard, subs);
                BridgeType::Option {
                    inner: Some(Box::new(inner)),
                }
            }
            AstType::Union(types) => {
                let mut parts = Vec::new();
                let mut saw_null = false;
                for part in *types {
                    let converted = self.convert_type_internal(part, alias_guard, subs);
                    if is_null_type(&converted) {
                        saw_null = true;
                    } else {
                        parts.push(converted);
                    }
                }
                if saw_null && parts.len() == 1 {
                    BridgeType::Option {
                        inner: Some(Box::new(parts.remove(0))),
                    }
                } else {
                    if saw_null {
                        parts.push(BridgeType::Primitive {
                            name: "null".to_string(),
                        });
                    }
                    BridgeType::Union { types: parts }
                }
            }
            AstType::Intersection(types) => {
                // Intersection types are not supported in the bridge yet; fall back to mixed.
                let _ = types;
                BridgeType::Mixed
            }
            AstType::ObjectShape(fields) => {
                let mut out = Vec::new();
                for field in *fields {
                    let name = token_text(self.source, field.name);
                    let ty = self.convert_type_internal(field.ty, alias_guard, subs);
                    out.push(BridgeField {
                        name,
                        ty,
                        optional: field.optional,
                    });
                }
                BridgeType::ObjectShape { fields: out }
            }
            AstType::Applied { base, args } => {
                let base_name = self.type_name(base).unwrap_or_else(|| "unknown".to_string());
                if let Some(alias) = self.aliases.get(&base_name).cloned() {
                    if alias.params.len() == args.len() {
                        let mut param_map = HashMap::new();
                        for (idx, param) in alias.params.iter().enumerate() {
                            let arg_ty = self.convert_type_internal(&args[idx], alias_guard, subs);
                            param_map.insert(param.clone(), arg_ty);
                        }
                        if alias_guard.insert(base_name.clone()) {
                            let resolved =
                                self.convert_type_internal(alias.ty, alias_guard, Some(&param_map));
                            alias_guard.remove(&base_name);
                            return resolved;
                        }
                        return BridgeType::Mixed;
                    }
                }
                let base_id = Self::base_name(&base_name).to_ascii_lowercase();
                let mut converted_args = Vec::new();
                for arg in *args {
                    converted_args.push(self.convert_type_internal(arg, alias_guard, subs));
                }
                if base_id == "option" {
                    return BridgeType::Option {
                        inner: converted_args.get(0).cloned().map(Box::new),
                    };
                }
                if base_id == "result" {
                    let ok = converted_args.get(0).cloned().map(Box::new);
                    let err = converted_args.get(1).cloned().map(Box::new);
                    return BridgeType::Result { ok, err };
                }
                if base_id == "array" {
                    let element = converted_args.get(0).cloned().map(Box::new);
                    return BridgeType::Array { element };
                }
                BridgeType::Applied {
                    base: base_name,
                    args: converted_args,
                }
            }
        }
    }

    fn convert_named(
        &mut self,
        name: &str,
        alias_guard: &mut HashSet<String>,
    ) -> Option<BridgeType> {
        let base = Self::base_name(name).to_ascii_lowercase();
        match base.as_str() {
            "mixed" => return Some(BridgeType::Mixed),
            "int" | "float" | "bool" | "string" | "null" => {
                return Some(BridgeType::Primitive {
                    name: base.to_string(),
                })
            }
            "array" => return Some(BridgeType::Array { element: None }),
            "object" => return Some(BridgeType::Object),
            "option" => {
                return Some(BridgeType::Option { inner: None });
            }
            "result" => {
                return Some(BridgeType::Result {
                    ok: None,
                    err: None,
                });
            }
            _ => {}
        }

        if let Some(fields) = self.structs.get(name).cloned() {
            return Some(BridgeType::Struct {
                name: name.to_string(),
                fields,
            });
        }

        if let Some(alias_type) = self.aliases.get(name) {
            if !alias_type.params.is_empty() {
                return Some(BridgeType::Mixed);
            }
            if alias_guard.insert(name.to_string()) {
                let resolved = self.convert_type_internal(alias_type.ty, alias_guard, None);
                alias_guard.remove(name);
                return Some(resolved);
            }
            return Some(BridgeType::Mixed);
        }

        Some(BridgeType::Unknown)
    }
}

fn is_null_type(ty: &BridgeType) -> bool {
    matches!(ty, BridgeType::Primitive { name } if name == "null")
}

fn token_text(source: &[u8], token: &Token) -> String {
    String::from_utf8_lossy(token.text(source)).to_string()
}

fn name_to_string(source: &[u8], name: &php_rs::parser::ast::Name<'_>) -> String {
    let mut out = String::new();
    for (idx, part) in name.parts.iter().enumerate() {
        if idx > 0 {
            out.push('\\');
        }
        out.push_str(&token_text(source, part));
    }
    out
}

fn collect_aliases<'a>(
    program: &'a Program<'a>,
    source: &'a [u8],
) -> HashMap<String, TypeAliasInfo<'a>> {
    let mut out = HashMap::new();
    for stmt in program.statements.iter() {
        if let Stmt::TypeAlias {
            name,
            type_params,
            ty,
            ..
        } = stmt
        {
            let name_str = token_text(source, name);
            let params = type_params
                .iter()
                .map(|param| token_text(source, param.name))
                .collect::<Vec<_>>();
            out.insert(
                name_str,
                TypeAliasInfo {
                    params,
                    ty: *ty,
                },
            );
        }
    }
    out
}

fn collect_structs<'a>(
    program: &'a Program<'a>,
    source: &'a [u8],
    aliases: &HashMap<String, TypeAliasInfo<'a>>,
) -> HashMap<String, Vec<BridgeField>> {
    let mut out = HashMap::new();
    for stmt in program.statements.iter() {
        let Stmt::Class { kind, name, members, .. } = stmt else {
            continue;
        };
        if *kind != ClassKind::Struct {
            continue;
        }
        let struct_name = token_text(source, name);
        let mut fields = Vec::new();
        for member in members.iter() {
            match *member {
                ClassMember::Property { ty, entries, .. } => {
                    for entry in entries.iter() {
                        let field_name = token_text(source, entry.name);
                        let optional = entry.default.is_some();
                        let field_ty = ty
                            .map(|ty| {
                                let mut resolver = TypeResolver {
                                    source,
                                    aliases: aliases.clone(),
                                    structs: HashMap::new(),
                                };
                                resolver.convert_type(ty)
                            })
                            .unwrap_or(BridgeType::Mixed);
                        fields.push(BridgeField {
                            name: field_name,
                            ty: field_ty,
                            optional,
                        });
                    }
                }
                _ => {}
            }
        }
        out.insert(struct_name, fields);
    }
    out
}

#[op2]
#[serde]
fn op_php_parse_phpx_types(
    #[string] source: String,
    #[string] file_path: String,
) -> Result<BridgeModuleTypes, deno_core::error::CoreError> {
    let mut cleaned_source = None;
    let trimmed = source.trim_start();
    let source_bytes = if trimmed.starts_with("<?php") {
        let prefix_len = source.len() - trimmed.len();
        let without_tag = source[prefix_len + 5..].to_string();
        cleaned_source = Some(without_tag);
        cleaned_source.as_ref().unwrap().as_bytes()
    } else {
        source.as_bytes()
    };
    let arena = Bump::new();
    let lexer = Lexer::new(source_bytes);
    let mut parser = Parser::new_with_mode(lexer, &arena, ParserMode::Phpx);
    let program = parser.parse_program();
    if !program.errors.is_empty() {
        return Err(deno_core::error::CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!(
                "Failed to parse PHPX types for '{}': {:?}",
                file_path, program.errors
            ),
        )));
    }
    let mut resolver = TypeResolver::new(source_bytes, &program);
    let mut functions = HashMap::new();
    for stmt in program.statements.iter() {
        if let Stmt::Function {
            name,
            params,
            return_type,
            ..
        } = stmt
        {
            let fn_name = token_text(source_bytes, name);
            let mut params_out = Vec::new();
            let mut has_variadic = false;
            for param in params.iter() {
                let ty = param.ty.map(|ty| resolver.convert_type(ty));
                let required = param.default.is_none() && !param.variadic;
                let variadic = param.variadic;
                if variadic {
                    has_variadic = true;
                }
                params_out.push(BridgeParam {
                    ty,
                    required,
                    variadic,
                });
            }
            let return_type = return_type.map(|ty| resolver.convert_type(ty));
            functions.insert(
                fn_name,
                BridgeFunction {
                    params: params_out,
                    return_type,
                    variadic: has_variadic,
                },
            );
        }
    }
    Ok(BridgeModuleTypes {
        functions,
        structs: resolver
            .structs
            .iter()
            .map(|(k, v)| (k.clone(), BridgeStruct { fields: v.clone() }))
            .collect(),
    })
}

#[op2]
#[buffer]
fn op_php_get_wasm() -> Vec<u8> {
    PHP_WASM_BYTES.to_vec()
}

#[op2]
#[buffer]
fn op_php_read_file_sync(#[string] path: String) -> Result<Vec<u8>, deno_core::error::CoreError> {
    std::fs::read(&path).map_err(|e| {
        deno_core::error::CoreError::from(std::io::Error::new(
            e.kind(),
            format!("Failed to read file '{}': {}", path, e),
        ))
    })
}

#[op2(fast)]
fn op_php_write_file_sync(
    #[string] path: String,
    #[buffer] data: &[u8],
) -> Result<(), deno_core::error::CoreError> {
    std::fs::write(&path, data).map_err(|e| {
        deno_core::error::CoreError::from(std::io::Error::new(
            e.kind(),
            format!("Failed to write file '{}': {}", path, e),
        ))
    })
}

#[op2(fast)]
fn op_php_mkdirs(#[string] path: String) -> Result<(), deno_core::error::CoreError> {
    std::fs::create_dir_all(&path).map_err(|e| {
        deno_core::error::CoreError::from(std::io::Error::new(
            e.kind(),
            format!("Failed to create dir '{}': {}", path, e),
        ))
    })
}

#[op2]
#[string]
fn op_php_sha256(#[string] data: String) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    let digest = hasher.finalize();
    format!("{:x}", digest)
}

#[op2]
#[serde]
fn op_php_read_env() -> HashMap<String, String> {
    let mut merged = HashMap::new();
    for (key, value) in std::env::vars() {
        merged.insert(key, value);
    }
    for (key, value) in read_dotenv_from_cwd() {
        merged.insert(key, value);
    }
    merged
}

fn read_dotenv_from_cwd() -> HashMap<String, String> {
    let out = HashMap::new();
    let Ok(cwd) = std::env::current_dir() else {
        return out;
    };
    let path = cwd.join(".env");
    let Ok(raw) = std::fs::read_to_string(path) else {
        return out;
    };
    parse_dotenv(&raw)
}

fn parse_dotenv(raw: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let body = trimmed.strip_prefix("export ").unwrap_or(trimmed);
        let Some((key_raw, value_raw)) = body.split_once('=') else {
            continue;
        };
        let key = key_raw.trim();
        if key.is_empty() {
            continue;
        }
        let mut value = value_raw.trim().to_string();
        if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
            value = decode_double_quoted(&value[1..value.len() - 1]);
        } else if value.starts_with('\'') && value.ends_with('\'') && value.len() >= 2 {
            value = value[1..value.len() - 1].to_string();
        } else if let Some(idx) = value.find(" #") {
            value = value[..idx].trim_end().to_string();
        }
        out.insert(key.to_string(), value);
    }
    out
}

fn decode_double_quoted(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        let Some(next) = chars.next() else {
            out.push('\\');
            break;
        };
        match next {
            'n' => out.push('\n'),
            'r' => out.push('\r'),
            't' => out.push('\t'),
            '"' => out.push('"'),
            '\\' => out.push('\\'),
            other => {
                out.push('\\');
                out.push(other);
            }
        }
    }
    out
}

fn db_call_impl(
    action: String,
    args: serde_json::Value,
) -> Result<serde_json::Value, deno_core::error::CoreError> {
    let err = |msg: String| {
        deno_core::error::CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            msg,
        ))
    };

    let args_obj = args.as_object().cloned().unwrap_or_default();
    match action.as_str() {
        "open" => {
            let started = Instant::now();
            let raw_driver = args_obj
                .get("driver")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let driver = raw_driver
                .split('\0')
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            let cfg = args_obj
                .get("config")
                .and_then(|v| v.as_object())
                .cloned()
                .unwrap_or_default();

            let (key, driver_cfg) = match driver.as_str() {
                d if d.starts_with("postgres") => {
                    let host = cfg
                        .get("host")
                        .and_then(|v| v.as_str())
                        .unwrap_or("127.0.0.1")
                        .trim_matches('\0')
                        .to_string();
                    let port = cfg.get("port").and_then(|v| v.as_u64()).unwrap_or(5432);
                    let user = cfg
                        .get("user")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .trim_matches('\0')
                        .to_string();
                    let password = cfg
                        .get("password")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .trim_matches('\0')
                        .to_string();
                    let database = cfg
                        .get("database")
                        .and_then(|v| v.as_str())
                        .or_else(|| cfg.get("dbname").and_then(|v| v.as_str()))
                        .unwrap_or("")
                        .trim_matches('\0')
                        .to_string();
                    (
                        format!(
                            "postgres://{}:{}@{}:{}/{}",
                            user, password, host, port, database
                        ),
                        DbDriverConfig::Postgres(PgConnConfig {
                            host,
                            port: port as u16,
                            database,
                            user,
                            password,
                        }),
                    )
                }
                d if d.starts_with("sqlite") => {
                    let path = cfg
                        .get("path")
                        .and_then(|v| v.as_str())
                        .or_else(|| cfg.get("file").and_then(|v| v.as_str()))
                        .unwrap_or("")
                        .trim_matches('\0')
                        .to_string();
                    if path.is_empty() {
                        return Ok(serde_json::json!({
                            "ok": false,
                            "error": "sqlite open requires config.path"
                        }));
                    }
                    (
                        format!("sqlite://{}", path),
                        DbDriverConfig::Sqlite(SqliteConnConfig { path }),
                    )
                }
                d if d.starts_with("mysql") => {
                    let host = cfg
                        .get("host")
                        .and_then(|v| v.as_str())
                        .unwrap_or("127.0.0.1")
                        .trim_matches('\0')
                        .to_string();
                    let port = cfg.get("port").and_then(|v| v.as_u64()).unwrap_or(3306);
                    let user = cfg
                        .get("user")
                        .and_then(|v| v.as_str())
                        .unwrap_or("root")
                        .trim_matches('\0')
                        .to_string();
                    let password = cfg
                        .get("password")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .trim_matches('\0')
                        .to_string();
                    let database = cfg
                        .get("database")
                        .and_then(|v| v.as_str())
                        .or_else(|| cfg.get("dbname").and_then(|v| v.as_str()))
                        .unwrap_or("")
                        .trim_matches('\0')
                        .to_string();
                    (
                        format!("mysql://{}:{}@{}:{}/{}", user, password, host, port, database),
                        DbDriverConfig::Mysql(MysqlConnConfig {
                            host,
                            port: port as u16,
                            database,
                            user,
                            password,
                        }),
                    )
                }
                _ => {
                    return Ok(serde_json::json!({
                        "ok": false,
                        "error": format!("unsupported driver '{}'", driver)
                    }));
                }
            };

            {
                let state = db_state()
                    .lock()
                    .map_err(|_| err("db lock poisoned".to_string()))?;
                if let Some(handle) = state.key_to_handle.get(&key).copied() {
                    drop(state);
                    if let Ok(mut state) = db_state().lock() {
                        state.record_metric("open", &driver, started.elapsed().as_millis() as u64, false);
                    }
                    return Ok(serde_json::json!({
                        "ok": true,
                        "handle": handle,
                        "reused": true
                    }));
                }
            }

            let mut state = db_state()
                .lock()
                .map_err(|_| err("db lock poisoned".to_string()))?;
            if let Some(handle) = state.key_to_handle.get(&key).copied() {
                state.record_metric("open", &driver, started.elapsed().as_millis() as u64, false);
                return Ok(serde_json::json!({
                    "ok": true,
                    "handle": handle,
                    "reused": true
                }));
            }
            let handle = state.next_handle;
            state.next_handle += 1;
            state.handles.insert(
                handle,
                DbConn {
                    key: key.clone(),
                    config: driver_cfg,
                },
            );
            state.key_to_handle.insert(key, handle);
            state.record_metric("open", &driver, started.elapsed().as_millis() as u64, false);
            Ok(serde_json::json!({
                "ok": true,
                "handle": handle,
                "reused": false
            }))
        }
        "query" => {
            let started = Instant::now();
            let handle = args_obj
                .get("handle")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| err("query: missing handle".to_string()))?;
            let sql = args_obj
                .get("sql")
                .and_then(|v| v.as_str())
                .ok_or_else(|| err("query: missing sql".to_string()))?;
            let params = args_obj
                .get("params")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();

            let driver_cfg = {
                let state = db_state()
                    .lock()
                    .map_err(|_| err("db lock poisoned".to_string()))?;
                let conn = state
                    .handles
                    .get(&handle)
                    .ok_or_else(|| err(format!("query: unknown handle {}", handle)))?;
                conn.config.clone()
            };
            let driver_name = driver_cfg.driver_name();
            let sql = sql.to_string();
            let out_rows_result = match driver_cfg {
                DbDriverConfig::Postgres(cfg) => with_pg_client(cfg, move |client| {
                    let boxed: Vec<Box<dyn ToSql + Sync>> =
                        params.iter().map(json_to_pg_param).collect();
                    let refs: Vec<&(dyn ToSql + Sync)> = boxed.iter().map(|v| v.as_ref()).collect();
                    let rows = client.query(&sql, &refs).map_err(|e| {
                        deno_core::error::CoreError::from(std::io::Error::other(format!(
                            "postgres query failed: {}",
                            e
                        )))
                    })?;

                    let mut out_rows = Vec::with_capacity(rows.len());
                    for row in &rows {
                        let mut obj = serde_json::Map::new();
                        for idx in 0..row.len() {
                            let name = row.columns()[idx].name().to_string();
                            obj.insert(name, pg_cell_to_json(row, idx));
                        }
                        out_rows.push(serde_json::Value::Object(obj));
                    }
                    Ok(out_rows)
                })?,
                DbDriverConfig::Sqlite(cfg) => with_sqlite_conn(cfg, move |conn| {
                    let mut stmt = conn.prepare(&sql).map_err(|e| {
                        deno_core::error::CoreError::from(std::io::Error::other(format!(
                            "sqlite prepare failed: {}",
                            e
                        )))
                    })?;
                    let sqlite_params: Vec<rusqlite::types::Value> =
                        params.iter().map(json_to_sqlite_value).collect();
                    let mut rows = stmt
                        .query(sqlite_params_from_iter(sqlite_params.iter()))
                        .map_err(|e| {
                            deno_core::error::CoreError::from(std::io::Error::other(format!(
                                "sqlite query failed: {}",
                                e
                            )))
                        })?;

                    let mut out_rows = Vec::new();
                    while let Some(row) = rows.next().map_err(|e| {
                        deno_core::error::CoreError::from(std::io::Error::other(format!(
                            "sqlite row fetch failed: {}",
                            e
                        )))
                    })? {
                        let mut obj = serde_json::Map::new();
                        let row_ref = row.as_ref();
                        for idx in 0..row_ref.column_count() {
                            let name = row_ref.column_name(idx).unwrap_or("").to_string();
                            obj.insert(name, sqlite_cell_to_json(row, idx));
                        }
                        out_rows.push(serde_json::Value::Object(obj));
                    }
                    Ok(out_rows)
                })?,
                DbDriverConfig::Mysql(cfg) => with_mysql_conn(cfg, move |conn| {
                    let mysql_params =
                        MyParams::Positional(params.iter().map(json_to_mysql_value).collect());
                    let rows: Vec<mysql::Row> = conn.exec(&sql, mysql_params).map_err(|e| {
                        deno_core::error::CoreError::from(std::io::Error::other(format!(
                            "mysql query failed: {}",
                            e
                        )))
                    })?;

                    let mut out_rows = Vec::with_capacity(rows.len());
                    for row in &rows {
                        let mut obj = serde_json::Map::new();
                        let cols = row.columns_ref();
                        for (idx, col) in cols.iter().enumerate() {
                            let name = col.name_str().to_string();
                            let value = row
                                .as_ref(idx)
                                .map(mysql_value_to_json)
                                .unwrap_or(serde_json::Value::Null);
                            obj.insert(name, value);
                        }
                        out_rows.push(serde_json::Value::Object(obj));
                    }
                    Ok(out_rows)
                })?,
            };
            let elapsed_ms = started.elapsed().as_millis() as u64;
            let mut metric_state = db_state()
                .lock()
                .map_err(|_| err("db lock poisoned".to_string()))?;
            metric_state.record_metric("query", driver_name, elapsed_ms, false);
            drop(metric_state);
            let out_rows = out_rows_result;

            Ok(serde_json::json!({
                "ok": true,
                "rows": out_rows
            }))
        }
        "exec" => {
            let started = Instant::now();
            let handle = args_obj
                .get("handle")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| err("exec: missing handle".to_string()))?;
            let sql = args_obj
                .get("sql")
                .and_then(|v| v.as_str())
                .ok_or_else(|| err("exec: missing sql".to_string()))?;
            let params = args_obj
                .get("params")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();

            let driver_cfg = {
                let state = db_state()
                    .lock()
                    .map_err(|_| err("db lock poisoned".to_string()))?;
                let conn = state
                    .handles
                    .get(&handle)
                    .ok_or_else(|| err(format!("exec: unknown handle {}", handle)))?;
                conn.config.clone()
            };
            let driver_name = driver_cfg.driver_name();
            let sql = sql.to_string();
            let affected_result = match driver_cfg {
                DbDriverConfig::Postgres(cfg) => with_pg_client(cfg, move |client| {
                    let boxed: Vec<Box<dyn ToSql + Sync>> =
                        params.iter().map(json_to_pg_param).collect();
                    let refs: Vec<&(dyn ToSql + Sync)> = boxed.iter().map(|v| v.as_ref()).collect();
                    client.execute(&sql, &refs).map_err(|e| {
                        deno_core::error::CoreError::from(std::io::Error::other(format!(
                            "postgres exec failed: {}",
                            e
                        )))
                    })
                })?,
                DbDriverConfig::Sqlite(cfg) => with_sqlite_conn(cfg, move |conn| {
                    let mut stmt = conn.prepare(&sql).map_err(|e| {
                        deno_core::error::CoreError::from(std::io::Error::other(format!(
                            "sqlite prepare failed: {}",
                            e
                        )))
                    })?;
                    let sqlite_params: Vec<rusqlite::types::Value> =
                        params.iter().map(json_to_sqlite_value).collect();
                    let changed = stmt
                        .execute(sqlite_params_from_iter(sqlite_params.iter()))
                        .map_err(|e| {
                            deno_core::error::CoreError::from(std::io::Error::other(format!(
                                "sqlite exec failed: {}",
                                e
                            )))
                        })?;
                    Ok(changed as u64)
                })?,
                DbDriverConfig::Mysql(cfg) => with_mysql_conn(cfg, move |conn| {
                    let mysql_params =
                        MyParams::Positional(params.iter().map(json_to_mysql_value).collect());
                    let result = conn.exec_iter(&sql, mysql_params).map_err(|e| {
                        deno_core::error::CoreError::from(std::io::Error::other(format!(
                            "mysql exec failed: {}",
                            e
                        )))
                    })?;
                    Ok(result.affected_rows())
                })?,
            };
            let elapsed_ms = started.elapsed().as_millis() as u64;
            let mut metric_state = db_state()
                .lock()
                .map_err(|_| err("db lock poisoned".to_string()))?;
            metric_state.record_metric("exec", driver_name, elapsed_ms, false);
            drop(metric_state);
            let affected = affected_result;

            Ok(serde_json::json!({
                "ok": true,
                "affected_rows": affected
            }))
        }
        "begin" => {
            // Transaction scope across multiple calls is not supported in stateless mode.
            Ok(serde_json::json!({ "ok": true }))
        }
        "commit" => {
            Ok(serde_json::json!({ "ok": true }))
        }
        "rollback" => {
            Ok(serde_json::json!({ "ok": true }))
        }
        "close" => {
            let started = Instant::now();
            let handle = args_obj
                .get("handle")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| err("close: missing handle".to_string()))?;
            let mut state = db_state()
                .lock()
                .map_err(|_| err("db lock poisoned".to_string()))?;
            if let Some(conn) = state.handles.remove(&handle) {
                state.record_metric(
                    "close",
                    conn.config.driver_name(),
                    started.elapsed().as_millis() as u64,
                    false,
                );
                state.key_to_handle.remove(&conn.key);
            }
            Ok(serde_json::json!({ "ok": true }))
        }
        "stats" => {
            let state = db_state()
                .lock()
                .map_err(|_| err("db lock poisoned".to_string()))?;
            let mut handles_by_driver: HashMap<String, u64> = HashMap::new();
            for conn in state.handles.values() {
                let key = conn.config.driver_name().to_string();
                let prev = handles_by_driver.get(&key).copied().unwrap_or(0);
                handles_by_driver.insert(key, prev + 1);
            }

            let mut metrics = serde_json::Map::new();
            for (key, metric) in &state.metrics {
                let avg_ms = if metric.calls == 0 {
                    0
                } else {
                    metric.total_ms / metric.calls
                };
                metrics.insert(
                    key.clone(),
                    serde_json::json!({
                        "calls": metric.calls,
                        "errors": metric.errors,
                        "total_ms": metric.total_ms,
                        "avg_ms": avg_ms
                    }),
                );
            }

            Ok(serde_json::json!({
                "ok": true,
                "active_handles": state.handles.len() as u64,
                "handles_by_driver": handles_by_driver,
                "metrics": metrics
            }))
        }
        _ => Ok(serde_json::json!({
            "ok": false,
            "error": format!("unknown db action '{}'", action)
        })),
    }
}

#[derive(Clone, Copy)]
enum DbProtoActionKind {
    Open,
    Query,
    QueryOne,
    Exec,
    Begin,
    Commit,
    Rollback,
    Close,
    Stats,
}

fn core_err(msg: impl Into<String>) -> deno_core::error::CoreError {
    deno_core::error::CoreError::from(std::io::Error::other(msg.into()))
}

fn db_json_to_proto_value(value: &serde_json::Value) -> proto::bridge_v1::Value {
    use proto::bridge_v1::value::Kind;
    let kind = match value {
        serde_json::Value::Null => Some(Kind::NullValue(proto::bridge_v1::NullValue {})),
        serde_json::Value::Bool(v) => Some(Kind::BoolValue(*v)),
        serde_json::Value::Number(v) => {
            if let Some(i) = v.as_i64() {
                Some(Kind::IntValue(i))
            } else if let Some(f) = v.as_f64() {
                Some(Kind::FloatValue(f))
            } else {
                Some(Kind::StringValue(v.to_string()))
            }
        }
        serde_json::Value::String(v) => Some(Kind::StringValue(v.clone())),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            Some(Kind::StringValue(value.to_string()))
        }
    };
    proto::bridge_v1::Value { kind }
}

fn db_proto_to_json_value(value: &proto::bridge_v1::Value) -> serde_json::Value {
    use proto::bridge_v1::value::Kind;
    match value.kind.as_ref() {
        Some(Kind::NullValue(_)) => serde_json::Value::Null,
        Some(Kind::BoolValue(v)) => serde_json::Value::Bool(*v),
        Some(Kind::IntValue(v)) => serde_json::Value::Number((*v).into()),
        Some(Kind::FloatValue(v)) => serde_json::Number::from_f64(*v)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        Some(Kind::StringValue(v)) => serde_json::Value::String(v.clone()),
        Some(Kind::BytesValue(v)) => serde_json::Value::Array(
            v.iter()
                .map(|b| serde_json::Value::Number((*b as u64).into()))
                .collect(),
        ),
        None => serde_json::Value::Null,
    }
}

fn db_proto_request_to_action_payload(
    req: &proto::bridge_v1::DbRequest,
) -> Result<(String, serde_json::Value, DbProtoActionKind), deno_core::error::CoreError> {
    use proto::bridge_v1::db_request::Action;
    let Some(action) = req.action.as_ref() else {
        return Err(core_err("db proto request missing action"));
    };
    match action {
        Action::Open(open) => {
            let mut cfg = serde_json::Map::new();
            for item in &open.config {
                let value = item
                    .value
                    .as_ref()
                    .map(db_proto_to_json_value)
                    .unwrap_or(serde_json::Value::Null);
                cfg.insert(item.key.clone(), value);
            }
            Ok((
                "open".to_string(),
                serde_json::json!({
                    "driver": open.driver,
                    "config": cfg
                }),
                DbProtoActionKind::Open,
            ))
        }
        Action::Query(query) => Ok((
            "query".to_string(),
            serde_json::json!({
                "handle": query.handle,
                "sql": query.sql,
                "params": query.params.iter().map(db_proto_to_json_value).collect::<Vec<_>>()
            }),
            DbProtoActionKind::Query,
        )),
        Action::QueryOne(query) => Ok((
            "query".to_string(),
            serde_json::json!({
                "handle": query.handle,
                "sql": query.sql,
                "params": query.params.iter().map(db_proto_to_json_value).collect::<Vec<_>>()
            }),
            DbProtoActionKind::QueryOne,
        )),
        Action::Exec(exec) => Ok((
            "exec".to_string(),
            serde_json::json!({
                "handle": exec.handle,
                "sql": exec.sql,
                "params": exec.params.iter().map(db_proto_to_json_value).collect::<Vec<_>>()
            }),
            DbProtoActionKind::Exec,
        )),
        Action::Begin(h) => Ok((
            "begin".to_string(),
            serde_json::json!({ "handle": h.handle }),
            DbProtoActionKind::Begin,
        )),
        Action::Commit(h) => Ok((
            "commit".to_string(),
            serde_json::json!({ "handle": h.handle }),
            DbProtoActionKind::Commit,
        )),
        Action::Rollback(h) => Ok((
            "rollback".to_string(),
            serde_json::json!({ "handle": h.handle }),
            DbProtoActionKind::Rollback,
        )),
        Action::Close(h) => Ok((
            "close".to_string(),
            serde_json::json!({ "handle": h.handle }),
            DbProtoActionKind::Close,
        )),
        Action::Stats(_) => Ok((
            "stats".to_string(),
            serde_json::json!({}),
            DbProtoActionKind::Stats,
        )),
    }
}

fn db_action_payload_to_proto_request(
    action: &str,
    payload: &serde_json::Value,
) -> Result<proto::bridge_v1::DbRequest, deno_core::error::CoreError> {
    use proto::bridge_v1::db_request::Action;
    let args = payload.as_object().cloned().unwrap_or_default();
    let action = match action {
        "open" => {
            let driver = args
                .get("driver")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let mut config = Vec::new();
            if let Some(cfg) = args.get("config").and_then(|v| v.as_object()) {
                for (key, val) in cfg {
                    config.push(proto::bridge_v1::NamedValue {
                        key: key.clone(),
                        value: Some(db_json_to_proto_value(val)),
                    });
                }
            }
            Action::Open(proto::bridge_v1::DbOpenRequest { driver, config })
        }
        "query" => {
            let handle = args.get("handle").and_then(|v| v.as_u64()).unwrap_or(0);
            let sql = args
                .get("sql")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let params = args
                .get("params")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default()
                .iter()
                .map(db_json_to_proto_value)
                .collect();
            Action::Query(proto::bridge_v1::DbQueryRequest {
                handle,
                sql,
                params,
            })
        }
        "query_one" => {
            let handle = args.get("handle").and_then(|v| v.as_u64()).unwrap_or(0);
            let sql = args
                .get("sql")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let params = args
                .get("params")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default()
                .iter()
                .map(db_json_to_proto_value)
                .collect();
            Action::QueryOne(proto::bridge_v1::DbQueryRequest {
                handle,
                sql,
                params,
            })
        }
        "exec" => {
            let handle = args.get("handle").and_then(|v| v.as_u64()).unwrap_or(0);
            let sql = args
                .get("sql")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let params = args
                .get("params")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default()
                .iter()
                .map(db_json_to_proto_value)
                .collect();
            Action::Exec(proto::bridge_v1::DbExecRequest {
                handle,
                sql,
                params,
            })
        }
        "begin" => Action::Begin(proto::bridge_v1::DbHandleRequest {
            handle: args.get("handle").and_then(|v| v.as_u64()).unwrap_or(0),
        }),
        "commit" => Action::Commit(proto::bridge_v1::DbHandleRequest {
            handle: args.get("handle").and_then(|v| v.as_u64()).unwrap_or(0),
        }),
        "rollback" => Action::Rollback(proto::bridge_v1::DbHandleRequest {
            handle: args.get("handle").and_then(|v| v.as_u64()).unwrap_or(0),
        }),
        "close" => Action::Close(proto::bridge_v1::DbHandleRequest {
            handle: args.get("handle").and_then(|v| v.as_u64()).unwrap_or(0),
        }),
        "stats" => Action::Stats(true),
        other => {
            return Err(core_err(format!("unsupported db proto action '{}'", other)));
        }
    };

    Ok(proto::bridge_v1::DbRequest {
        schema_version: 1,
        action: Some(action),
    })
}

fn db_json_rows_to_proto_rows(value: &serde_json::Value) -> Vec<proto::bridge_v1::Row> {
    let Some(rows) = value.as_array() else {
        return Vec::new();
    };
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        if let Some(obj) = row.as_object() {
            let fields = obj
                .iter()
                .map(|(key, val)| proto::bridge_v1::NamedValue {
                    key: key.clone(),
                    value: Some(db_json_to_proto_value(val)),
                })
                .collect();
            out.push(proto::bridge_v1::Row { fields });
        }
    }
    out
}

fn db_proto_rows_to_json(rows: &[proto::bridge_v1::Row]) -> serde_json::Value {
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let mut obj = serde_json::Map::new();
        for field in &row.fields {
            let val = field
                .value
                .as_ref()
                .map(db_proto_to_json_value)
                .unwrap_or(serde_json::Value::Null);
            obj.insert(field.key.clone(), val);
        }
        out.push(serde_json::Value::Object(obj));
    }
    serde_json::Value::Array(out)
}

fn db_json_response_to_proto(
    resp: &serde_json::Value,
    kind: DbProtoActionKind,
) -> proto::bridge_v1::DbResponse {
    use proto::bridge_v1::db_response::Action;
    let ok = resp.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
    let error = resp
        .get("error")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let action = match kind {
        DbProtoActionKind::Open => {
            let handle = resp.get("handle").and_then(|v| v.as_u64()).unwrap_or(0);
            let reused = resp.get("reused").and_then(|v| v.as_bool()).unwrap_or(false);
            Some(Action::Open(proto::bridge_v1::DbOpenResponse { handle, reused }))
        }
        DbProtoActionKind::Query => {
            let rows = db_json_rows_to_proto_rows(resp.get("rows").unwrap_or(&serde_json::Value::Null));
            Some(Action::Query(proto::bridge_v1::DbRowsResponse { rows }))
        }
        DbProtoActionKind::QueryOne => {
            let mut rows = db_json_rows_to_proto_rows(resp.get("rows").unwrap_or(&serde_json::Value::Null));
            if rows.len() > 1 {
                rows.truncate(1);
            }
            Some(Action::QueryOne(proto::bridge_v1::DbRowsResponse { rows }))
        }
        DbProtoActionKind::Exec => {
            let affected_rows = resp
                .get("affected_rows")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            Some(Action::Exec(proto::bridge_v1::DbExecResponse { affected_rows }))
        }
        DbProtoActionKind::Begin => Some(Action::Begin(proto::bridge_v1::DbUnitResponse { ok })),
        DbProtoActionKind::Commit => Some(Action::Commit(proto::bridge_v1::DbUnitResponse { ok })),
        DbProtoActionKind::Rollback => {
            Some(Action::Rollback(proto::bridge_v1::DbUnitResponse { ok }))
        }
        DbProtoActionKind::Close => Some(Action::Close(proto::bridge_v1::DbUnitResponse { ok })),
        DbProtoActionKind::Stats => {
            let active_handles = resp
                .get("active_handles")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            let mut handles_by_driver = Vec::new();
            if let Some(obj) = resp.get("handles_by_driver").and_then(|v| v.as_object()) {
                for (driver, count_val) in obj {
                    let count = count_val.as_u64().unwrap_or(0);
                    handles_by_driver.push(proto::bridge_v1::DriverCount {
                        driver: driver.clone(),
                        count,
                    });
                }
            }

            let mut metrics = Vec::new();
            if let Some(obj) = resp.get("metrics").and_then(|v| v.as_object()) {
                for (key, metric_val) in obj {
                    let calls = metric_val
                        .get("calls")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let errors = metric_val
                        .get("errors")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let total_ms = metric_val
                        .get("total_ms")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let avg_ms = metric_val
                        .get("avg_ms")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    metrics.push(proto::bridge_v1::NamedMetric {
                        key: key.clone(),
                        metric: Some(proto::bridge_v1::DbStatsMetric {
                            calls,
                            errors,
                            total_ms,
                            avg_ms,
                        }),
                    });
                }
            }

            Some(Action::Stats(proto::bridge_v1::DbStatsResponse {
                active_handles,
                handles_by_driver,
                metrics,
            }))
        }
    };

    proto::bridge_v1::DbResponse {
        schema_version: 1,
        ok,
        error,
        action,
    }
}

fn db_proto_response_to_json(resp: &proto::bridge_v1::DbResponse) -> serde_json::Value {
    use proto::bridge_v1::db_response::Action;
    let mut out = serde_json::Map::new();
    out.insert("ok".to_string(), serde_json::Value::Bool(resp.ok));
    if !resp.error.is_empty() {
        out.insert("error".to_string(), serde_json::Value::String(resp.error.clone()));
    }
    if let Some(action) = resp.action.as_ref() {
        match action {
            Action::Open(open) => {
                out.insert(
                    "handle".to_string(),
                    serde_json::Value::Number(open.handle.into()),
                );
                out.insert("reused".to_string(), serde_json::Value::Bool(open.reused));
            }
            Action::Query(rows) | Action::QueryOne(rows) => {
                out.insert("rows".to_string(), db_proto_rows_to_json(&rows.rows));
            }
            Action::Exec(exec) => {
                out.insert(
                    "affected_rows".to_string(),
                    serde_json::Value::Number(exec.affected_rows.into()),
                );
            }
            Action::Begin(unit)
            | Action::Commit(unit)
            | Action::Rollback(unit)
            | Action::Close(unit) => {
                out.insert("ok".to_string(), serde_json::Value::Bool(unit.ok));
            }
            Action::Stats(stats) => {
                out.insert(
                    "active_handles".to_string(),
                    serde_json::Value::Number(stats.active_handles.into()),
                );
                let mut by_driver = serde_json::Map::new();
                for item in &stats.handles_by_driver {
                    by_driver.insert(item.driver.clone(), serde_json::Value::Number(item.count.into()));
                }
                out.insert("handles_by_driver".to_string(), serde_json::Value::Object(by_driver));

                let mut metrics = serde_json::Map::new();
                for metric in &stats.metrics {
                    let m = metric.metric.as_ref();
                    metrics.insert(
                        metric.key.clone(),
                        serde_json::json!({
                            "calls": m.map(|x| x.calls).unwrap_or(0),
                            "errors": m.map(|x| x.errors).unwrap_or(0),
                            "total_ms": m.map(|x| x.total_ms).unwrap_or(0),
                            "avg_ms": m.map(|x| x.avg_ms).unwrap_or(0),
                        }),
                    );
                }
                out.insert("metrics".to_string(), serde_json::Value::Object(metrics));
            }
        }
    }
    serde_json::Value::Object(out)
}

#[op2]
#[serde]
fn op_php_db_call(
    #[string] action: String,
    #[serde] args: serde_json::Value,
) -> Result<serde_json::Value, deno_core::error::CoreError> {
    db_call_impl(action, args)
}

fn db_call_proto_impl(request: &[u8]) -> Result<Vec<u8>, deno_core::error::CoreError> {
    let req = proto::bridge_v1::DbRequest::decode(request)
        .map_err(|e| core_err(format!("db proto decode failed: {}", e)))?;
    let (action, payload, kind) = db_proto_request_to_action_payload(&req)?;
    let response_json = db_call_impl(action, payload)?;
    let response = db_json_response_to_proto(&response_json, kind);
    Ok(response.encode_to_vec())
}

#[op2]
#[buffer]
fn op_php_db_call_proto(
    #[buffer] request: &[u8],
) -> Result<Vec<u8>, deno_core::error::CoreError> {
    db_call_proto_impl(request)
}

#[op2]
#[buffer]
fn op_php_db_proto_encode(
    #[string] action: String,
    #[serde] payload: serde_json::Value,
) -> Result<Vec<u8>, deno_core::error::CoreError> {
    let request = db_action_payload_to_proto_request(&action, &payload)?;
    Ok(request.encode_to_vec())
}

#[op2]
#[serde]
fn op_php_db_proto_decode(
    #[buffer] response: &[u8],
) -> Result<serde_json::Value, deno_core::error::CoreError> {
    let decoded = proto::bridge_v1::DbResponse::decode(response)
        .map_err(|e| core_err(format!("db proto decode response failed: {}", e)))?;
    Ok(db_proto_response_to_json(&decoded))
}

#[op2]
#[serde]
fn op_php_net_call(
    #[string] action: String,
    #[serde] args: serde_json::Value,
) -> Result<serde_json::Value, deno_core::error::CoreError> {
    let err = |msg: String| {
        deno_core::error::CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            msg,
        ))
    };

    let args_obj = args.as_object().cloned().unwrap_or_default();
    match action.as_str() {
        "connect" => {
            let host = args_obj
                .get("host")
                .and_then(|v| v.as_str())
                .unwrap_or("127.0.0.1")
                .trim_matches('\0')
                .to_string();
            let port = args_obj
                .get("port")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u16;
            if port == 0 {
                return Ok(serde_json::json!({ "ok": false, "error": "connect: missing or invalid port" }));
            }
            let timeout_ms = args_obj
                .get("timeout_ms")
                .and_then(|v| v.as_u64())
                .unwrap_or(5000);
            let addr = format!("{}:{}", host, port);
            let mut addrs = addr
                .to_socket_addrs()
                .map_err(|e| err(format!("connect: resolve failed: {}", e)))?;
            let target = addrs
                .next()
                .ok_or_else(|| err("connect: no resolved address".to_string()))?;
            let stream = TcpStream::connect_timeout(&target, Duration::from_millis(timeout_ms))
                .map_err(|e| err(format!("connect: {}", e)))?;
            let mut state = net_state()
                .lock()
                .map_err(|_| err("net lock poisoned".to_string()))?;
            let handle = state.next_handle;
            state.next_handle += 1;
            state.handles.insert(handle, NetConn::Tcp(stream));
            Ok(serde_json::json!({ "ok": true, "handle": handle }))
        }
        "set_deadline" => {
            let handle = args_obj
                .get("handle")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| err("set_deadline: missing handle".to_string()))?;
            let millis = args_obj
                .get("millis")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let timeout = if millis == 0 {
                None
            } else {
                Some(Duration::from_millis(millis))
            };
            let mut state = net_state()
                .lock()
                .map_err(|_| err("net lock poisoned".to_string()))?;
            let Some(conn) = state.handles.get_mut(&handle) else {
                return Ok(serde_json::json!({ "ok": false, "error": format!("set_deadline: unknown handle {}", handle) }));
            };
            let result = match conn {
                NetConn::Tcp(stream) => stream
                    .set_read_timeout(timeout)
                    .and_then(|_| stream.set_write_timeout(timeout)),
                NetConn::Tls(stream) => stream
                    .get_ref()
                    .set_read_timeout(timeout)
                    .and_then(|_| stream.get_ref().set_write_timeout(timeout)),
            };
            match result {
                Ok(()) => Ok(serde_json::json!({ "ok": true })),
                Err(e) => Ok(serde_json::json!({ "ok": false, "error": format!("set_deadline: {}", e) })),
            }
        }
        "read" => {
            let handle = args_obj
                .get("handle")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| err("read: missing handle".to_string()))?;
            let max_bytes = args_obj
                .get("max_bytes")
                .and_then(|v| v.as_u64())
                .unwrap_or(4096) as usize;
            let mut buf = vec![0_u8; max_bytes.max(1)];
            let mut state = net_state()
                .lock()
                .map_err(|_| err("net lock poisoned".to_string()))?;
            let Some(conn) = state.handles.get_mut(&handle) else {
                return Ok(serde_json::json!({ "ok": false, "error": format!("read: unknown handle {}", handle) }));
            };
            let n = match conn {
                NetConn::Tcp(stream) => stream.read(&mut buf),
                NetConn::Tls(stream) => stream.read(&mut buf),
            };
            match n {
                Ok(n) => {
                    let data = String::from_utf8_lossy(&buf[..n]).to_string();
                    Ok(serde_json::json!({ "ok": true, "data": data, "eof": n == 0 }))
                }
                Err(e) => Ok(serde_json::json!({ "ok": false, "error": format!("read: {}", e) })),
            }
        }
        "write" => {
            let handle = args_obj
                .get("handle")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| err("write: missing handle".to_string()))?;
            let data = args_obj
                .get("data")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .as_bytes()
                .to_vec();
            let mut state = net_state()
                .lock()
                .map_err(|_| err("net lock poisoned".to_string()))?;
            let Some(conn) = state.handles.get_mut(&handle) else {
                return Ok(serde_json::json!({ "ok": false, "error": format!("write: unknown handle {}", handle) }));
            };
            let result = match conn {
                NetConn::Tcp(stream) => stream.write_all(&data),
                NetConn::Tls(stream) => stream.write_all(&data),
            };
            match result {
                Ok(()) => Ok(serde_json::json!({ "ok": true, "written": data.len() })),
                Err(e) => Ok(serde_json::json!({ "ok": false, "error": format!("write: {}", e) })),
            }
        }
        "tls_upgrade" => {
            let handle = args_obj
                .get("handle")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| err("tls_upgrade: missing handle".to_string()))?;
            let server_name = args_obj
                .get("server_name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim_matches('\0')
                .to_string();
            if server_name.is_empty() {
                return Ok(serde_json::json!({ "ok": false, "error": "tls_upgrade: missing server_name" }));
            }
            let mut state = net_state()
                .lock()
                .map_err(|_| err("net lock poisoned".to_string()))?;
            let Some(conn) = state.handles.remove(&handle) else {
                return Ok(serde_json::json!({ "ok": false, "error": format!("tls_upgrade: unknown handle {}", handle) }));
            };
            let tcp = match conn {
                NetConn::Tcp(stream) => stream,
                NetConn::Tls(stream) => {
                    let new_handle = state.next_handle;
                    state.next_handle += 1;
                    state.handles.insert(new_handle, NetConn::Tls(stream));
                    return Ok(serde_json::json!({ "ok": true, "handle": new_handle, "reused": true }));
                }
            };
            let connector = TlsConnector::new()
                .map_err(|e| err(format!("tls_upgrade: connector init failed: {}", e)))?;
            match connector.connect(&server_name, tcp) {
                Ok(stream) => {
                    let new_handle = state.next_handle;
                    state.next_handle += 1;
                    state.handles.insert(new_handle, NetConn::Tls(stream));
                    Ok(serde_json::json!({ "ok": true, "handle": new_handle }))
                }
                Err(e) => Ok(serde_json::json!({ "ok": false, "error": format!("tls_upgrade: {}", e) })),
            }
        }
        "close" => {
            let handle = args_obj
                .get("handle")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| err("close: missing handle".to_string()))?;
            let mut state = net_state()
                .lock()
                .map_err(|_| err("net lock poisoned".to_string()))?;
            state.handles.remove(&handle);
            Ok(serde_json::json!({ "ok": true }))
        }
        _ => Ok(serde_json::json!({
            "ok": false,
            "error": format!("unknown net action '{}'", action)
        })),
    }
}

fn fs_call_impl(
    action: String,
    args: serde_json::Value,
) -> Result<serde_json::Value, deno_core::error::CoreError> {
    let err = |msg: String| {
        deno_core::error::CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            msg,
        ))
    };

    let to_bytes = |value: Option<&serde_json::Value>| -> Vec<u8> {
        let Some(value) = value else {
            return Vec::new();
        };
        if let Some(arr) = value.as_array() {
            let mut out = Vec::with_capacity(arr.len());
            for item in arr {
                let byte = item.as_u64().unwrap_or(0).min(255) as u8;
                out.push(byte);
            }
            return out;
        }
        if let Some(s) = value.as_str() {
            return s.as_bytes().to_vec();
        }
        Vec::new()
    };

    let args_obj = args.as_object().cloned().unwrap_or_default();
    match action.as_str() {
        "open" => {
            let path = args_obj
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim_matches('\0')
                .to_string();
            if path.is_empty() {
                return Ok(serde_json::json!({ "ok": false, "error": "open: missing path" }));
            }

            let mode = args_obj
                .get("mode")
                .and_then(|v| v.as_str())
                .unwrap_or("r");
            let mut opts = OpenOptions::new();
            let read = mode.contains('r') || mode.contains('+');
            let write = mode.contains('w') || mode.contains('a') || mode.contains('x') || mode.contains('c') || mode.contains('+');
            let append = mode.contains('a');
            let truncate = mode.contains('w');
            let create = mode.contains('w') || mode.contains('a') || mode.contains('x') || mode.contains('c');
            let create_new = mode.contains('x');

            opts.read(read)
                .write(write)
                .append(append)
                .truncate(truncate)
                .create(create)
                .create_new(create_new);

            let file = match opts.open(&path) {
                Ok(file) => file,
                Err(e) => {
                    return Ok(serde_json::json!({
                        "ok": false,
                        "error": format!("open: {}", e)
                    }));
                }
            };

            let mut state = fs_state()
                .lock()
                .map_err(|_| err("fs lock poisoned".to_string()))?;
            let handle = state.next_handle;
            state.next_handle += 1;
            state.handles.insert(handle, file);
            Ok(serde_json::json!({ "ok": true, "handle": handle }))
        }
        "read" => {
            let handle = args_obj
                .get("handle")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| err("read: missing handle".to_string()))?;
            let max_bytes = args_obj
                .get("max_bytes")
                .and_then(|v| v.as_u64())
                .unwrap_or(65536) as usize;
            let mut buf = vec![0_u8; max_bytes.max(1)];

            let mut state = fs_state()
                .lock()
                .map_err(|_| err("fs lock poisoned".to_string()))?;
            let Some(file) = state.handles.get_mut(&handle) else {
                return Ok(serde_json::json!({
                    "ok": false,
                    "error": format!("read: unknown handle {}", handle)
                }));
            };

            match file.read(&mut buf) {
                Ok(n) => {
                    buf.truncate(n);
                    Ok(serde_json::json!({
                        "ok": true,
                        "data": buf,
                        "eof": n == 0
                    }))
                }
                Err(e) => Ok(serde_json::json!({
                    "ok": false,
                    "error": format!("read: {}", e)
                })),
            }
        }
        "write" => {
            let handle = args_obj
                .get("handle")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| err("write: missing handle".to_string()))?;
            let data = to_bytes(args_obj.get("data"));

            let mut state = fs_state()
                .lock()
                .map_err(|_| err("fs lock poisoned".to_string()))?;
            let Some(file) = state.handles.get_mut(&handle) else {
                return Ok(serde_json::json!({
                    "ok": false,
                    "error": format!("write: unknown handle {}", handle)
                }));
            };

            match file.write_all(&data) {
                Ok(()) => Ok(serde_json::json!({ "ok": true, "written": data.len() })),
                Err(e) => Ok(serde_json::json!({
                    "ok": false,
                    "error": format!("write: {}", e)
                })),
            }
        }
        "close" => {
            let handle = args_obj
                .get("handle")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| err("close: missing handle".to_string()))?;
            let mut state = fs_state()
                .lock()
                .map_err(|_| err("fs lock poisoned".to_string()))?;
            state.handles.remove(&handle);
            Ok(serde_json::json!({ "ok": true }))
        }
        "read_file" => {
            let path = args_obj
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim_matches('\0')
                .to_string();
            if path.is_empty() {
                return Ok(serde_json::json!({ "ok": false, "error": "read_file: missing path" }));
            }
            match std::fs::read(&path) {
                Ok(data) => Ok(serde_json::json!({ "ok": true, "data": data })),
                Err(e) => Ok(serde_json::json!({
                    "ok": false,
                    "error": format!("read_file: {}", e)
                })),
            }
        }
        "write_file" => {
            let path = args_obj
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim_matches('\0')
                .to_string();
            if path.is_empty() {
                return Ok(serde_json::json!({ "ok": false, "error": "write_file: missing path" }));
            }
            let data = to_bytes(args_obj.get("data"));
            match std::fs::write(&path, &data) {
                Ok(()) => Ok(serde_json::json!({ "ok": true, "written": data.len() })),
                Err(e) => Ok(serde_json::json!({
                    "ok": false,
                    "error": format!("write_file: {}", e)
                })),
            }
        }
        _ => Ok(serde_json::json!({
            "ok": false,
            "error": format!("unknown fs action '{}'", action)
        })),
    }
}

#[derive(Clone, Copy)]
enum FsProtoActionKind {
    Open,
    Read,
    Write,
    Close,
    ReadFile,
    WriteFile,
}

fn fs_action_payload_to_proto_request(
    action: &str,
    payload: &serde_json::Value,
) -> Result<proto::bridge_v1::FsRequest, deno_core::error::CoreError> {
    use proto::bridge_v1::fs_request::Action;
    let args = payload.as_object().cloned().unwrap_or_default();
    let action = match action {
        "open" => Action::Open(proto::bridge_v1::FsOpenRequest {
            path: args
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            mode: args
                .get("mode")
                .and_then(|v| v.as_str())
                .unwrap_or("r")
                .to_string(),
        }),
        "read" => Action::Read(proto::bridge_v1::FsReadRequest {
            handle: args.get("handle").and_then(|v| v.as_u64()).unwrap_or(0),
            max_bytes: args.get("max_bytes").and_then(|v| v.as_u64()).unwrap_or(65536),
        }),
        "write" => {
            let data = args
                .get("data")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .map(|x| x.as_u64().unwrap_or(0).min(255) as u8)
                        .collect::<Vec<u8>>()
                })
                .or_else(|| args.get("data").and_then(|v| v.as_str()).map(|s| s.as_bytes().to_vec()))
                .unwrap_or_default();
            Action::Write(proto::bridge_v1::FsWriteRequest {
                handle: args.get("handle").and_then(|v| v.as_u64()).unwrap_or(0),
                data,
            })
        }
        "close" => Action::Close(proto::bridge_v1::FsCloseRequest {
            handle: args.get("handle").and_then(|v| v.as_u64()).unwrap_or(0),
        }),
        "read_file" => Action::ReadFile(proto::bridge_v1::FsReadFileRequest {
            path: args
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        }),
        "write_file" => {
            let data = args
                .get("data")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .map(|x| x.as_u64().unwrap_or(0).min(255) as u8)
                        .collect::<Vec<u8>>()
                })
                .or_else(|| args.get("data").and_then(|v| v.as_str()).map(|s| s.as_bytes().to_vec()))
                .unwrap_or_default();
            Action::WriteFile(proto::bridge_v1::FsWriteFileRequest {
                path: args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                data,
            })
        }
        other => return Err(core_err(format!("unsupported fs proto action '{}'", other))),
    };
    Ok(proto::bridge_v1::FsRequest {
        schema_version: 1,
        action: Some(action),
    })
}

fn fs_proto_request_to_action_payload(
    req: &proto::bridge_v1::FsRequest,
) -> Result<(String, serde_json::Value, FsProtoActionKind), deno_core::error::CoreError> {
    use proto::bridge_v1::fs_request::Action;
    let Some(action) = req.action.as_ref() else {
        return Err(core_err("fs proto request missing action"));
    };
    match action {
        Action::Open(open) => Ok((
            "open".to_string(),
            serde_json::json!({ "path": open.path, "mode": open.mode }),
            FsProtoActionKind::Open,
        )),
        Action::Read(read) => Ok((
            "read".to_string(),
            serde_json::json!({ "handle": read.handle, "max_bytes": read.max_bytes }),
            FsProtoActionKind::Read,
        )),
        Action::Write(write) => Ok((
            "write".to_string(),
            serde_json::json!({
                "handle": write.handle,
                "data": write.data.iter().map(|b| serde_json::Value::Number((*b as u64).into())).collect::<Vec<_>>()
            }),
            FsProtoActionKind::Write,
        )),
        Action::Close(close) => Ok((
            "close".to_string(),
            serde_json::json!({ "handle": close.handle }),
            FsProtoActionKind::Close,
        )),
        Action::ReadFile(read_file) => Ok((
            "read_file".to_string(),
            serde_json::json!({ "path": read_file.path }),
            FsProtoActionKind::ReadFile,
        )),
        Action::WriteFile(write_file) => Ok((
            "write_file".to_string(),
            serde_json::json!({
                "path": write_file.path,
                "data": write_file.data.iter().map(|b| serde_json::Value::Number((*b as u64).into())).collect::<Vec<_>>()
            }),
            FsProtoActionKind::WriteFile,
        )),
    }
}

fn fs_json_response_to_proto(
    resp: &serde_json::Value,
    kind: FsProtoActionKind,
) -> proto::bridge_v1::FsResponse {
    use proto::bridge_v1::fs_response::Action;
    let ok = resp.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
    let error = resp
        .get("error")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let action = match kind {
        FsProtoActionKind::Open => Some(Action::Open(proto::bridge_v1::FsOpenResponse {
            handle: resp.get("handle").and_then(|v| v.as_u64()).unwrap_or(0),
        })),
        FsProtoActionKind::Read => {
            let data = resp
                .get("data")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .map(|x| x.as_u64().unwrap_or(0).min(255) as u8)
                        .collect::<Vec<u8>>()
                })
                .unwrap_or_default();
            Some(Action::Read(proto::bridge_v1::FsReadResponse {
                data,
                eof: resp.get("eof").and_then(|v| v.as_bool()).unwrap_or(false),
            }))
        }
        FsProtoActionKind::Write => Some(Action::Write(proto::bridge_v1::FsWriteResponse {
            written: resp.get("written").and_then(|v| v.as_u64()).unwrap_or(0),
        })),
        FsProtoActionKind::Close => Some(Action::Close(proto::bridge_v1::FsUnitResponse { ok })),
        FsProtoActionKind::ReadFile => {
            let data = resp
                .get("data")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .map(|x| x.as_u64().unwrap_or(0).min(255) as u8)
                        .collect::<Vec<u8>>()
                })
                .unwrap_or_default();
            Some(Action::ReadFile(proto::bridge_v1::FsReadResponse { data, eof: true }))
        }
        FsProtoActionKind::WriteFile => {
            Some(Action::WriteFile(proto::bridge_v1::FsWriteResponse {
                written: resp.get("written").and_then(|v| v.as_u64()).unwrap_or(0),
            }))
        }
    };
    proto::bridge_v1::FsResponse {
        schema_version: 1,
        ok,
        error,
        action,
    }
}

fn fs_proto_response_to_json(resp: &proto::bridge_v1::FsResponse) -> serde_json::Value {
    use proto::bridge_v1::fs_response::Action;
    let mut out = serde_json::Map::new();
    out.insert("ok".to_string(), serde_json::Value::Bool(resp.ok));
    if !resp.error.is_empty() {
        out.insert("error".to_string(), serde_json::Value::String(resp.error.clone()));
    }
    if let Some(action) = resp.action.as_ref() {
        match action {
            Action::Open(open) => {
                out.insert("handle".to_string(), serde_json::Value::Number(open.handle.into()));
            }
            Action::Read(read) | Action::ReadFile(read) => {
                out.insert(
                    "data".to_string(),
                    serde_json::Value::Array(
                        read.data
                            .iter()
                            .map(|b| serde_json::Value::Number((*b as u64).into()))
                            .collect(),
                    ),
                );
                out.insert("eof".to_string(), serde_json::Value::Bool(read.eof));
            }
            Action::Write(write) | Action::WriteFile(write) => {
                out.insert(
                    "written".to_string(),
                    serde_json::Value::Number(write.written.into()),
                );
            }
            Action::Close(unit) => {
                out.insert("ok".to_string(), serde_json::Value::Bool(unit.ok));
            }
        }
    }
    serde_json::Value::Object(out)
}

fn fs_call_proto_impl(request: &[u8]) -> Result<Vec<u8>, deno_core::error::CoreError> {
    let req = proto::bridge_v1::FsRequest::decode(request)
        .map_err(|e| core_err(format!("fs proto decode failed: {}", e)))?;
    let (action, payload, kind) = fs_proto_request_to_action_payload(&req)?;
    let response_json = fs_call_impl(action, payload)?;
    let response = fs_json_response_to_proto(&response_json, kind);
    Ok(response.encode_to_vec())
}

#[op2]
#[serde]
fn op_php_fs_call(
    #[string] action: String,
    #[serde] args: serde_json::Value,
) -> Result<serde_json::Value, deno_core::error::CoreError> {
    fs_call_impl(action, args)
}

#[op2]
#[buffer]
fn op_php_fs_call_proto(
    #[buffer] request: &[u8],
) -> Result<Vec<u8>, deno_core::error::CoreError> {
    fs_call_proto_impl(request)
}

#[op2]
#[buffer]
fn op_php_fs_proto_encode(
    #[string] action: String,
    #[serde] payload: serde_json::Value,
) -> Result<Vec<u8>, deno_core::error::CoreError> {
    let request = fs_action_payload_to_proto_request(&action, &payload)?;
    Ok(request.encode_to_vec())
}

#[op2]
#[serde]
fn op_php_fs_proto_decode(
    #[buffer] response: &[u8],
) -> Result<serde_json::Value, deno_core::error::CoreError> {
    let decoded = proto::bridge_v1::FsResponse::decode(response)
        .map_err(|e| core_err(format!("fs proto decode response failed: {}", e)))?;
    Ok(fs_proto_response_to_json(&decoded))
}

#[op2]
#[string]
fn op_php_cwd() -> Result<String, deno_core::error::CoreError> {
    std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| deno_core::error::CoreError::from(e))
}

#[op2(fast)]
fn op_php_file_exists(#[string] path: String) -> bool {
    std::path::Path::new(&path).exists()
}

#[op2]
#[string]
fn op_php_path_resolve(#[string] base: String, #[string] path: String) -> String {
    let base_path = std::path::Path::new(&base);
    let target_path = std::path::Path::new(&path);

    let resolved = if target_path.is_absolute() {
        target_path.to_path_buf()
    } else {
        base_path.join(target_path)
    };

    resolved.to_string_lossy().to_string()
}

#[op2]
#[serde]
fn op_php_read_dir(#[string] path: String) -> Result<Vec<PhpDirEntry>, deno_core::error::CoreError> {
    let entries = std::fs::read_dir(&path).map_err(|e| {
        deno_core::error::CoreError::from(std::io::Error::new(
            e.kind(),
            format!("Failed to read dir '{}': {}", path, e),
        ))
    })?;

    let mut out = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| {
            deno_core::error::CoreError::from(std::io::Error::new(
                e.kind(),
                format!("Failed to read dir entry in '{}': {}", path, e),
            ))
        })?;
        let file_type = entry.file_type().map_err(|e| {
            deno_core::error::CoreError::from(std::io::Error::new(
                e.kind(),
                format!("Failed to read dir entry type in '{}': {}", path, e),
            ))
        })?;
        out.push(PhpDirEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            is_dir: file_type.is_dir(),
            is_file: file_type.is_file(),
        });
    }
    Ok(out)
}

#[op2]
#[serde]
fn op_php_parse_wit(
    #[string] path: String,
    #[string] world: String,
) -> Result<WitSchema, deno_core::error::CoreError> {
    let mut resolve = Resolve::default();
    let (package_id, _) = resolve.push_path(&path).map_err(|err| {
        deno_core::error::CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to parse WIT '{}': {}", path, err),
        ))
    })?;

    let world_id = if world.trim().is_empty() {
        let package = &resolve.packages[package_id];
        if package.worlds.len() != 1 {
            return Err(deno_core::error::CoreError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "WIT package has {} worlds; set deka.json.world",
                    package.worlds.len()
                ),
            )));
        }
        *package
            .worlds
            .values()
            .next()
            .expect("worlds len checked")
    } else {
        resolve.select_world(package_id, Some(world.trim())).map_err(|err| {
            deno_core::error::CoreError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to select world '{}': {}", world, err),
            ))
        })?
    };

    let world = &resolve.worlds[world_id];
    let mut functions = Vec::new();
    let mut interfaces = Vec::new();

    for (key, item) in world.exports.iter() {
        match item {
            WorldItem::Function(func) => {
                let sig = build_function(&resolve, func);
                let name = world_key_name(key);
                functions.push(WitFunction { name, ..sig });
            }
            WorldItem::Interface { id, .. } => {
                let iface = &resolve.interfaces[*id];
                let iface_name = world_key_name(key);
                let mut iface_functions = Vec::new();
                for func in iface.functions.values() {
                    let sig = build_function(&resolve, func);
                    iface_functions.push(sig);
                }
                interfaces.push(WitInterface {
                    name: iface_name,
                    functions: iface_functions,
                });
            }
            WorldItem::Type(_) => {}
        }
    }

    Ok(WitSchema {
        world: world.name.clone(),
        functions,
        interfaces,
    })
}

deno_core::extension!(
    php_core,
    ops = [
        op_php_get_wasm,
        op_php_parse_phpx_types,
        op_php_read_file_sync,
        op_php_write_file_sync,
        op_php_mkdirs,
        op_php_sha256,
        op_php_read_env,
        op_php_db_call,
        op_php_db_call_proto,
        op_php_db_proto_encode,
        op_php_db_proto_decode,
        op_php_net_call,
        op_php_fs_call,
        op_php_fs_call_proto,
        op_php_fs_proto_encode,
        op_php_fs_proto_decode,
        op_php_cwd,
        op_php_file_exists,
        op_php_path_resolve,
        op_php_read_dir,
        op_php_parse_wit,
    ],
    esm_entry_point = "ext:php_core/php.js",
    esm = [dir "src/modules/php", "php.js"],
);

pub fn init() -> deno_core::Extension {
    php_core::init_ops_and_esm()
}

#[cfg(test)]
mod tests {
    use super::*;
    use prost::Message;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_suffix() -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        format!("{}_{}", std::process::id(), nanos)
    }

    fn assert_ok(value: &serde_json::Value) {
        assert_eq!(
            value.get("ok").and_then(|v| v.as_bool()),
            Some(true),
            "expected ok response, got: {}",
            value
        );
    }

    #[test]
    fn db_proto_open_parity_postgres_mysql_sqlite() {
        let suffix = unique_suffix();
        let cases = vec![
            (
                "postgres",
                serde_json::json!({
                    "host": "127.0.0.1",
                    "port": 5432,
                    "database": format!("db_proto_pg_{}", suffix),
                    "user": "u",
                    "password": "p",
                }),
            ),
            (
                "mysql",
                serde_json::json!({
                    "host": "127.0.0.1",
                    "port": 3306,
                    "database": format!("db_proto_my_{}", suffix),
                    "user": "u",
                    "password": "p",
                }),
            ),
            (
                "sqlite",
                serde_json::json!({
                    "path": format!("/tmp/db_proto_open_{}.sqlite", suffix),
                }),
            ),
        ];

        for (driver, config) in cases {
            let payload = serde_json::json!({
                "driver": driver,
                "config": config
            });

            let json_res = db_call_impl("open".to_string(), payload.clone()).expect("json open failed");
            assert_ok(&json_res);
            let json_handle = json_res
                .get("handle")
                .and_then(|v| v.as_u64())
                .expect("json open missing handle");

            let proto_req = db_action_payload_to_proto_request("open", &payload)
                .expect("proto request build failed");
            let proto_bytes = proto_req.encode_to_vec();
            let proto_resp_bytes =
                db_call_proto_impl(&proto_bytes).expect("proto open dispatch failed");
            let proto_resp =
                proto::bridge_v1::DbResponse::decode(proto_resp_bytes.as_slice()).expect("decode failed");
            let proto_json = db_proto_response_to_json(&proto_resp);
            assert_ok(&proto_json);
            let proto_handle = proto_json
                .get("handle")
                .and_then(|v| v.as_u64())
                .expect("proto open missing handle");

            let close_json = db_call_impl(
                "close".to_string(),
                serde_json::json!({ "handle": json_handle }),
            )
            .expect("json close failed");
            assert_ok(&close_json);
            if proto_handle != json_handle {
                let close_proto = db_call_impl(
                    "close".to_string(),
                    serde_json::json!({ "handle": proto_handle }),
                )
                .expect("proto handle close failed");
                assert_ok(&close_proto);
            }
        }
    }

    #[test]
    fn db_proto_sqlite_exec_query_parity() {
        let suffix = unique_suffix();
        let path = format!("/tmp/db_proto_query_{}.sqlite", suffix);
        let open_payload = serde_json::json!({
            "driver": "sqlite",
            "config": { "path": path }
        });
        let open_res = db_call_impl("open".to_string(), open_payload).expect("open failed");
        assert_ok(&open_res);
        let handle = open_res
            .get("handle")
            .and_then(|v| v.as_u64())
            .expect("missing handle");

        let setup_sql = vec![
            "create table if not exists packages (name text, downloads integer)",
            "delete from packages",
            "insert into packages(name, downloads) values ('db', 10), ('component', 20)",
        ];
        for sql in setup_sql {
            let exec_res = db_call_impl(
                "exec".to_string(),
                serde_json::json!({
                    "handle": handle,
                    "sql": sql,
                    "params": []
                }),
            )
            .expect("exec failed");
            assert_ok(&exec_res);
        }

        let json_query = db_call_impl(
            "query".to_string(),
            serde_json::json!({
                "handle": handle,
                "sql": "select name, downloads from packages order by downloads asc",
                "params": []
            }),
        )
        .expect("json query failed");
        assert_ok(&json_query);

        let proto_req = db_action_payload_to_proto_request(
            "query",
            &serde_json::json!({
                "handle": handle,
                "sql": "select name, downloads from packages order by downloads asc",
                "params": []
            }),
        )
        .expect("proto query request build failed");
        let proto_resp = db_call_proto_impl(&proto_req.encode_to_vec()).expect("proto query failed");
        let proto_decoded =
            proto::bridge_v1::DbResponse::decode(proto_resp.as_slice()).expect("decode failed");
        let proto_json = db_proto_response_to_json(&proto_decoded);
        assert_ok(&proto_json);

        assert_eq!(json_query.get("rows"), proto_json.get("rows"));

        let close_res =
            db_call_impl("close".to_string(), serde_json::json!({ "handle": handle }))
                .expect("close failed");
        assert_ok(&close_res);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn fs_proto_binary_roundtrip_integrity() {
        let suffix = unique_suffix();
        let path = format!("/tmp/fs_proto_roundtrip_{}.bin", suffix);
        let payload = serde_json::json!({
            "path": path,
            "data": [0, 1, 2, 10, 127, 128, 200, 255]
        });

        let json_write = fs_call_impl("write_file".to_string(), payload.clone()).expect("json write_file failed");
        assert_ok(&json_write);

        let proto_write_req = fs_action_payload_to_proto_request("write_file", &payload)
            .expect("fs proto write request build failed");
        let proto_write_resp =
            fs_call_proto_impl(&proto_write_req.encode_to_vec()).expect("fs proto write_file dispatch failed");
        let proto_write_decoded =
            proto::bridge_v1::FsResponse::decode(proto_write_resp.as_slice()).expect("fs decode write response failed");
        let proto_write_json = fs_proto_response_to_json(&proto_write_decoded);
        assert_ok(&proto_write_json);

        let json_read = fs_call_impl(
            "read_file".to_string(),
            serde_json::json!({ "path": path }),
        )
        .expect("json read_file failed");
        assert_ok(&json_read);

        let proto_read_req = fs_action_payload_to_proto_request(
            "read_file",
            &serde_json::json!({ "path": path }),
        )
        .expect("fs proto read request build failed");
        let proto_read_resp =
            fs_call_proto_impl(&proto_read_req.encode_to_vec()).expect("fs proto read_file dispatch failed");
        let proto_read_decoded =
            proto::bridge_v1::FsResponse::decode(proto_read_resp.as_slice()).expect("fs decode read response failed");
        let proto_read_json = fs_proto_response_to_json(&proto_read_decoded);
        assert_ok(&proto_read_json);

        assert_eq!(json_read.get("data"), proto_read_json.get("data"));
        let _ = std::fs::remove_file(path);
    }
}

fn world_key_name(key: &WorldKey) -> String {
    match key {
        WorldKey::Name(name) => name.clone(),
        WorldKey::Interface(id) => format!("interface_{}", id.index()),
    }
}

fn build_function(resolve: &Resolve, func: &wit_parser::Function) -> WitFunction {
    let params = func
        .params
        .iter()
        .map(|(name, ty)| WitParam {
            name: name.clone(),
            ty: resolve_type(resolve, ty, &mut HashSet::new()),
        })
        .collect::<Vec<_>>();

    let result = match &func.results {
        Results::Anon(ty) => Some(resolve_type(resolve, ty, &mut HashSet::new())),
        Results::Named(named) => {
            if named.is_empty() {
                None
            } else if named.len() == 1 {
                Some(resolve_type(resolve, &named[0].1, &mut HashSet::new()))
            } else {
                let fields = named
                    .iter()
                    .map(|(name, ty)| WitField {
                        name: name.clone(),
                        ty: resolve_type(resolve, ty, &mut HashSet::new()),
                    })
                    .collect();
                Some(WitType::Record { fields })
            }
        }
    };

    WitFunction {
        name: func.name.clone(),
        params,
        result,
    }
}

fn resolve_type(resolve: &Resolve, ty: &Type, visiting: &mut HashSet<TypeId>) -> WitType {
    match ty {
        Type::Bool => WitType::Bool,
        Type::U8 => WitType::U8,
        Type::U16 => WitType::U16,
        Type::U32 => WitType::U32,
        Type::U64 => WitType::U64,
        Type::S8 => WitType::S8,
        Type::S16 => WitType::S16,
        Type::S32 => WitType::S32,
        Type::S64 => WitType::S64,
        Type::F32 => WitType::F32,
        Type::F64 => WitType::F64,
        Type::Char => WitType::Char,
        Type::String => WitType::String,
        Type::Id(id) => resolve_type_id(resolve, *id, visiting),
    }
}

fn resolve_type_id(resolve: &Resolve, id: TypeId, visiting: &mut HashSet<TypeId>) -> WitType {
    if !visiting.insert(id) {
        return WitType::Unsupported {
            detail: "recursive type".to_string(),
        };
    }
    let ty = &resolve.types[id];
    let out = match &ty.kind {
        TypeDefKind::Record(record) => WitType::Record {
            fields: record
                .fields
                .iter()
                .map(|field| WitField {
                    name: field.name.clone(),
                    ty: resolve_type(resolve, &field.ty, visiting),
                })
                .collect(),
        },
        TypeDefKind::Tuple(tuple) => WitType::Tuple {
            items: tuple
                .types
                .iter()
                .map(|ty| resolve_type(resolve, ty, visiting))
                .collect(),
        },
        TypeDefKind::Option(inner) => WitType::Option {
            some: Box::new(resolve_type(resolve, inner, visiting)),
        },
        TypeDefKind::Result(res) => WitType::Result {
            ok: res.ok.as_ref().map(|ty| Box::new(resolve_type(resolve, ty, visiting))),
            err: res
                .err
                .as_ref()
                .map(|ty| Box::new(resolve_type(resolve, ty, visiting))),
        },
        TypeDefKind::List(inner) => WitType::List {
            element: Box::new(resolve_type(resolve, inner, visiting)),
        },
        TypeDefKind::Enum(enm) => WitType::Enum {
            cases: enm.cases.iter().map(|c| c.name.clone()).collect(),
        },
        TypeDefKind::Flags(flags) => WitType::Flags {
            flags: flags.flags.iter().map(|f| f.name.clone()).collect(),
        },
        TypeDefKind::Variant(variant) => WitType::Variant {
            cases: variant
                .cases
                .iter()
                .map(|case| WitVariantCase {
                    name: case.name.clone(),
                    ty: case.ty.as_ref().map(|ty| resolve_type(resolve, ty, visiting)),
                })
                .collect(),
        },
        TypeDefKind::Type(inner) => resolve_type(resolve, inner, visiting),
        TypeDefKind::Resource => WitType::Resource,
        TypeDefKind::Handle(_)
        | TypeDefKind::Future(_)
        | TypeDefKind::Stream(_)
        | TypeDefKind::Unknown => WitType::Unsupported {
            detail: ty.kind.as_str().to_string(),
        },
    };
    visiting.remove(&id);
    out
}
