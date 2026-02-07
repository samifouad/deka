use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use bumpalo::Bump;
use core::{CommandSpec, Context, Registry, SubcommandSpec};
use php_rs::parser::ast::{ClassKind, ClassMember, Name, Stmt, Type as AstType};
use php_rs::parser::lexer::Lexer;
use php_rs::parser::parser::{Parser, ParserMode};
use postgres::{Client, NoTls};
use serde_json::json;
use stdio::{error, log};

const GENERATE: SubcommandSpec = SubcommandSpec {
    name: "generate",
    summary: "generate db client and migration artifacts from PHPX struct models",
    aliases: &["gen"],
    handler: cmd_generate,
};

const MIGRATE: SubcommandSpec = SubcommandSpec {
    name: "migrate",
    summary: "apply pending db migrations",
    aliases: &[],
    handler: cmd_migrate,
};

const INFO: SubcommandSpec = SubcommandSpec {
    name: "info",
    summary: "show db generation and migration state",
    aliases: &["status"],
    handler: cmd_info,
};

const FLUSH: SubcommandSpec = SubcommandSpec {
    name: "flush",
    summary: "reset database schema (dev only)",
    aliases: &[],
    handler: cmd_flush,
};

const SUBCOMMANDS: &[SubcommandSpec] = &[GENERATE, MIGRATE, INFO, FLUSH];

const COMMAND: CommandSpec = CommandSpec {
    name: "db",
    category: "database",
    summary: "database tooling for PHPX ORM generation and migrations",
    aliases: &[],
    subcommands: SUBCOMMANDS,
    handler: cmd,
};

pub fn register(registry: &mut Registry) {
    registry.add_command(COMMAND);
}

fn cmd(_context: &Context) {
    error(
        "db",
        "missing subcommand. use: deka db generate|migrate|info|flush",
    );
}

fn cmd_generate(context: &Context) {
    let cwd = &context.env.cwd;
    let source = match resolve_generate_input(cwd, context.args.positionals.first()) {
        Ok(path) => path,
        Err(message) => {
            error("db generate", &message);
            return;
        }
    };

    let source_text = match fs::read_to_string(&source) {
        Ok(value) => value,
        Err(err) => {
            error(
                "db generate",
                &format!("failed to read model entry {}: {}", source.display(), err),
            );
            return;
        }
    };

    let models = match extract_struct_models(&source_text, source.display().to_string()) {
        Ok(value) => value,
        Err(message) => {
            error("db generate", &message);
            return;
        }
    };
    if models.is_empty() {
        error(
            "db generate",
            &format!(
                "no struct models found in {}. define at least one `struct Name {{ ... }}`",
                source.display()
            ),
        );
        return;
    }

    let generated = match generate_db_artifacts(cwd, &source, &models) {
        Ok(value) => value,
        Err(message) => {
            error("db generate", &message);
            return;
        }
    };

    log(
        "db generate",
        &format!(
            "generated {} files from {} model(s) in {}",
            generated,
            models.len(),
            source.display()
        ),
    );
}

fn cmd_migrate(_context: &Context) {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let db_dir = cwd.join("db");
    let migrations_dir = db_dir.join("migrations");
    if !migrations_dir.is_dir() {
        error(
            "db migrate",
            "db/migrations directory not found. run `deka db generate <models>` first",
        );
        return;
    }

    let mut migration_files = match collect_migration_files(&migrations_dir) {
        Ok(value) => value,
        Err(message) => {
            error("db migrate", &message);
            return;
        }
    };
    migration_files.sort();
    if migration_files.is_empty() {
        log("db migrate", "no migration files found");
        return;
    }

    let conn = postgres_connection_string();
    let mut client = match Client::connect(&conn, NoTls) {
        Ok(value) => value,
        Err(err) => {
            error(
                "db migrate",
                &format!("failed to connect to postgres: {}", err),
            );
            return;
        }
    };

    if let Err(err) = ensure_migrations_table(&mut client) {
        error(
            "db migrate",
            &format!("failed to ensure migration table: {}", err),
        );
        return;
    }

    let applied = match load_applied_migrations(&mut client) {
        Ok(value) => value,
        Err(err) => {
            error(
                "db migrate",
                &format!("failed to read applied migrations: {}", err),
            );
            return;
        }
    };

    match apply_migrations(&mut client, &migration_files, &applied, "db migrate") {
        Ok((applied_now, skipped)) => {
            log(
                "db migrate",
                &format!("done: applied={}, skipped={}", applied_now, skipped),
            );
        }
        Err(message) => error("db migrate", &message),
    }
}

fn cmd_info(_context: &Context) {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let db_dir = cwd.join("db");
    let state_path = db_dir.join("_state.json");
    let migrations_dir = db_dir.join("migrations");

    if !state_path.is_file() {
        error(
            "db info",
            "db/_state.json not found. run `deka db generate <models>` first",
        );
        return;
    }

    let state_text = match fs::read_to_string(&state_path) {
        Ok(value) => value,
        Err(err) => {
            error(
                "db info",
                &format!("failed to read {}: {}", state_path.display(), err),
            );
            return;
        }
    };
    let parsed: serde_json::Value = match serde_json::from_str(&state_text) {
        Ok(value) => value,
        Err(err) => {
            error(
                "db info",
                &format!("failed to parse {}: {}", state_path.display(), err),
            );
            return;
        }
    };

    let model_count = parsed
        .get("model_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let source = parsed
        .get("source")
        .and_then(|v| v.as_str())
        .unwrap_or("<unknown>");
    let generated_at = parsed
        .get("generated_at_unix")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let migration_count = if migrations_dir.is_dir() {
        collect_migration_files(&migrations_dir)
            .map(|v| v.len())
            .unwrap_or(0)
    } else {
        0
    };

    log("db info", &format!("source: {}", source));
    log("db info", &format!("models: {}", model_count));
    log("db info", &format!("generated_at_unix: {}", generated_at));
    log("db info", &format!("migration_files: {}", migration_count));

    let mut applied_count = 0usize;
    let mut pending_count = migration_count;
    if let Ok(mut client) = Client::connect(&postgres_connection_string(), NoTls) {
        if ensure_migrations_table(&mut client).is_ok() {
            if let Ok(applied) = load_applied_migrations(&mut client) {
                applied_count = applied.len();
                pending_count = migration_count.saturating_sub(applied_count);
            }
        }
    }
    log("db info", &format!("applied_migrations: {}", applied_count));
    log("db info", &format!("pending_migrations: {}", pending_count));
}

fn cmd_flush(_context: &Context) {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let db_dir = cwd.join("db");
    let migrations_dir = db_dir.join("migrations");
    if !migrations_dir.is_dir() {
        error(
            "db flush",
            "db/migrations directory not found. run `deka db generate <models>` first",
        );
        return;
    }

    let mut migration_files = match collect_migration_files(&migrations_dir) {
        Ok(value) => value,
        Err(message) => {
            error("db flush", &message);
            return;
        }
    };
    migration_files.sort();

    let conn = postgres_connection_string();
    let mut client = match Client::connect(&conn, NoTls) {
        Ok(value) => value,
        Err(err) => {
            error(
                "db flush",
                &format!("failed to connect to postgres: {}", err),
            );
            return;
        }
    };

    if let Err(err) =
        client.batch_execute("DROP SCHEMA IF EXISTS public CASCADE; CREATE SCHEMA public;")
    {
        error("db flush", &format!("failed to reset schema: {}", err));
        return;
    }
    if let Err(err) = ensure_migrations_table(&mut client) {
        error(
            "db flush",
            &format!("failed to initialize migration table: {}", err),
        );
        return;
    }

    let none_applied = std::collections::HashSet::new();
    match apply_migrations(&mut client, &migration_files, &none_applied, "db flush") {
        Ok((applied_now, skipped)) => {
            log(
                "db flush",
                &format!(
                    "schema reset complete: applied={}, skipped={}",
                    applied_now, skipped
                ),
            );
        }
        Err(message) => error("db flush", &message),
    }
}

fn resolve_generate_input(cwd: &Path, input: Option<&String>) -> Result<PathBuf, String> {
    let raw = input
        .map(String::as_str)
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("types/index.phpx");
    let candidate = if let Some(stripped) = raw.strip_prefix("@/") {
        cwd.join(stripped)
    } else {
        cwd.join(raw)
    };
    resolve_model_entry(&candidate, raw)
}

fn resolve_model_entry(candidate: &Path, raw: &str) -> Result<PathBuf, String> {
    if candidate.is_file() {
        if candidate.extension().and_then(|v| v.to_str()) != Some("phpx") {
            return Err(format!("expected a .phpx model entry file, got: {}", raw));
        }
        return Ok(candidate.to_path_buf());
    }

    if candidate.is_dir() {
        let index = candidate.join("index.phpx");
        if index.is_file() {
            return Ok(index);
        }
        return Err(format!(
            "expected model entry file, got directory: {}. tried: {}/index.phpx",
            raw,
            raw.trim_end_matches('/')
        ));
    }

    Err(format!(
        "model input not found: {}. pass a .phpx file or a directory containing index.phpx",
        raw
    ))
}

const GENERATED_HEADER: &str = "/*\n\
 * AUTO-GENERATED FILE - DO NOT EDIT\n\
 * Generated by deka db generate\n\
 * Changes will be overwritten.\n\
 */\n\n";

#[derive(Debug, Clone)]
struct ModelDef {
    name: String,
    fields: Vec<FieldDef>,
}

#[derive(Debug, Clone)]
struct FieldDef {
    name: String,
    ty: String,
    annotations: Vec<FieldAnnotationDef>,
}

#[derive(Debug, Clone)]
struct FieldAnnotationDef {
    name: String,
    args: Vec<String>,
}

impl FieldDef {
    fn annotation(&self, name: &str) -> Option<&FieldAnnotationDef> {
        self.annotations.iter().find(|ann| ann.name == name)
    }

    fn has_annotation(&self, name: &str) -> bool {
        self.annotation(name).is_some()
    }

    fn mapped_name(&self) -> String {
        self.annotation("map")
            .and_then(|ann| ann.args.first())
            .map(|arg| unquote(arg))
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| self.name.clone())
    }
}

fn extract_struct_models(source: &str, file_path: String) -> Result<Vec<ModelDef>, String> {
    let arena = Bump::new();
    let mut parser = Parser::new_with_mode(Lexer::new(source.as_bytes()), &arena, ParserMode::Phpx);
    let program = parser.parse_program();
    if !program.errors.is_empty() {
        let rendered = program
            .errors
            .iter()
            .map(|err| err.to_human_readable_with_path(source.as_bytes(), Some(&file_path)))
            .collect::<Vec<_>>()
            .join("\n\n");
        return Err(rendered);
    }

    let mut out = Vec::new();
    collect_struct_models_from_statements(&mut out, program.statements, source.as_bytes());
    Ok(out)
}

fn collect_struct_models_from_statements(
    out: &mut Vec<ModelDef>,
    statements: &[&Stmt<'_>],
    source: &[u8],
) {
    for stmt in statements {
        match stmt {
            Stmt::Class {
                kind,
                name,
                members,
                ..
            } if *kind == ClassKind::Struct => {
                let mut fields = Vec::new();
                for member in *members {
                    if let ClassMember::Property { ty, entries, .. } = member {
                        let Some(field_type) = ty.as_ref() else {
                            continue;
                        };
                        for entry in *entries {
                            let field_name =
                                String::from_utf8_lossy(entry.name.text(source)).to_string();
                            fields.push(FieldDef {
                                name: field_name.trim_start_matches('$').to_string(),
                                ty: render_ast_type(field_type, source),
                                annotations: entry
                                    .annotations
                                    .iter()
                                    .map(|ann| FieldAnnotationDef {
                                        name: String::from_utf8_lossy(ann.name.text(source))
                                            .to_string(),
                                        args: ann
                                            .args
                                            .iter()
                                            .map(|arg| {
                                                String::from_utf8_lossy(arg.span().as_str(source))
                                                    .to_string()
                                            })
                                            .collect(),
                                    })
                                    .collect(),
                            });
                        }
                    }
                }
                out.push(ModelDef {
                    name: String::from_utf8_lossy(name.text(source)).to_string(),
                    fields,
                });
            }
            Stmt::Namespace {
                body: Some(body), ..
            } => {
                collect_struct_models_from_statements(out, body, source);
            }
            _ => {}
        }
    }
}

fn render_name(name: &Name<'_>, source: &[u8]) -> String {
    name.parts
        .iter()
        .map(|part| String::from_utf8_lossy(part.text(source)).to_string())
        .collect::<Vec<_>>()
        .join("\\")
}

fn render_ast_type(ty: &AstType<'_>, source: &[u8]) -> String {
    match ty {
        AstType::Simple(tok) => String::from_utf8_lossy(tok.text(source)).to_string(),
        AstType::Name(name) => render_name(name, source),
        AstType::Union(parts) => parts
            .iter()
            .map(|part| render_ast_type(part, source))
            .collect::<Vec<_>>()
            .join("|"),
        AstType::Intersection(parts) => parts
            .iter()
            .map(|part| render_ast_type(part, source))
            .collect::<Vec<_>>()
            .join("&"),
        AstType::Nullable(inner) => format!("?{}", render_ast_type(inner, source)),
        AstType::ObjectShape(fields) => {
            let rendered = fields
                .iter()
                .map(|field| {
                    let name = String::from_utf8_lossy(field.name.text(source)).to_string();
                    let optional = if field.optional { "?" } else { "" };
                    format!(
                        "{}{}: {}",
                        name,
                        optional,
                        render_ast_type(field.ty, source)
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("{{ {} }}", rendered)
        }
        AstType::Applied { base, args } => format!(
            "{}<{}>",
            render_ast_type(base, source),
            args.iter()
                .map(|arg| render_ast_type(arg, source))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn generate_db_artifacts(cwd: &Path, source: &Path, models: &[ModelDef]) -> Result<usize, String> {
    let db_dir = cwd.join("db");
    let generated_dir = db_dir.join(".generated");
    let migrations_dir = db_dir.join("migrations");
    fs::create_dir_all(&generated_dir)
        .map_err(|e| format!("failed to create db/.generated: {}", e))?;
    fs::create_dir_all(&migrations_dir)
        .map_err(|e| format!("failed to create db/migrations: {}", e))?;

    let index_path = db_dir.join("index.phpx");
    let client_path = db_dir.join("client.phpx");
    let meta_path = db_dir.join("meta.phpx");
    let state_path = db_dir.join("_state.json");
    let migration_path = migrations_dir.join("0001_init.sql");
    let schema_path = generated_dir.join("schema.json");

    let index_body = render_index_phpx();
    let client_body = render_client_phpx(models);
    let meta_body = render_meta_phpx(models);
    let state_body = render_state_json(source, models);
    let migration_body = render_init_migration(models);
    let schema_body = render_generated_schema_json(models);

    fs::write(&index_path, index_body)
        .map_err(|e| format!("failed to write {}: {}", index_path.display(), e))?;
    fs::write(&client_path, client_body)
        .map_err(|e| format!("failed to write {}: {}", client_path.display(), e))?;
    fs::write(&meta_path, meta_body)
        .map_err(|e| format!("failed to write {}: {}", meta_path.display(), e))?;
    fs::write(&state_path, state_body)
        .map_err(|e| format!("failed to write {}: {}", state_path.display(), e))?;
    fs::write(&migration_path, migration_body)
        .map_err(|e| format!("failed to write {}: {}", migration_path.display(), e))?;
    fs::write(&schema_path, schema_body)
        .map_err(|e| format!("failed to write {}: {}", schema_path.display(), e))?;

    Ok(6)
}

fn render_index_phpx() -> String {
    format!(
        "{}import {{ createClient }} from './client'\nimport {{ Meta }} from './meta'\n\nexport const db = createClient(Meta)\n",
        GENERATED_HEADER
    )
}

fn render_client_phpx(models: &[ModelDef]) -> String {
    let mut model_map = String::new();
    for model in models {
        model_map.push_str(&format!(
            "        '{}': {{ name: '{}' }},\n",
            model.name, model.name
        ));
    }
    format!(
        "{}import {{ open_handle }} from 'db'\n\n\
export function eq($column, $value) {{\n\
    return {{ kind: 'eq', column: $column, value: $value }}\n\
}}\n\n\
export function and(...$parts) {{\n\
    return {{ kind: 'and', parts: $parts }}\n\
}}\n\n\
export function or(...$parts) {{\n\
    return {{ kind: 'or', parts: $parts }}\n\
}}\n\n\
export function createClient($meta) {{\n\
    function connect($driver, $config) {{\n\
        return open_handle($driver, $config)\n\
    }}\n\n\
    function not_implemented($method) {{\n\
        return {{ ok: false, error: 'db client method not implemented yet: ' . $method }}\n\
    }}\n\n\
    return {{\n\
        $meta: $meta,\n\
        $models: {{\n{}        }},\n\
        connect: connect,\n\
        select: function() {{ return not_implemented('select') }},\n\
        insert: function($model) {{ return not_implemented('insert') }},\n\
        update: function($model) {{ return not_implemented('update') }},\n\
        delete: function($model) {{ return not_implemented('delete') }},\n\
        transaction: function($fn) {{ return not_implemented('transaction') }},\n\
        eq: eq,\n\
        and: and,\n\
        or: or\n\
    }}\n\
}}\n",
        GENERATED_HEADER, model_map
    )
}

fn render_meta_phpx(models: &[ModelDef]) -> String {
    let mut body = String::new();
    body.push_str(GENERATED_HEADER);
    body.push_str("export const Meta = {\n    models: {\n");
    for model in models {
        let table = to_table_name(&model.name);
        body.push_str(&format!("        {}: {{\n", model.name));
        body.push_str(&format!("            name: '{}',\n", model.name));
        body.push_str(&format!("            table: '{}',\n", table));
        body.push_str("            fields: {\n");
        for field in &model.fields {
            body.push_str(&format!(
                "                {}: {{ type: '{}', db_name: '{}', annotations: [{}] }},\n",
                field.name,
                field.ty,
                field.mapped_name(),
                field
                    .annotations
                    .iter()
                    .map(|ann| {
                        if ann.args.is_empty() {
                            format!("{{ name: '{}' }}", ann.name)
                        } else {
                            format!(
                                "{{ name: '{}', args: [{}] }}",
                                ann.name,
                                ann.args
                                    .iter()
                                    .map(|arg| format!("'{}'", arg.replace('\'', "\\'")))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            )
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        body.push_str("            }\n");
        body.push_str("        },\n");
    }
    body.push_str("    }\n}\n");
    body
}

fn render_state_json(source: &Path, models: &[ModelDef]) -> String {
    let generated_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let payload = json!({
        "version": 1,
        "source": source.display().to_string(),
        "generated_at_unix": generated_at,
        "model_count": models.len(),
        "models": models.iter().map(|m| m.name.clone()).collect::<Vec<_>>(),
    });
    serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
}

fn render_generated_schema_json(models: &[ModelDef]) -> String {
    let payload = json!({
        "version": 1,
        "models": models
            .iter()
            .map(|model| {
                json!({
                    "name": model.name,
                    "table": to_table_name(&model.name),
                    "fields": model.fields
                        .iter()
                        .map(|field| {
                            let (sql_type, nullable) = map_sql_type(&field.ty);
                            json!({
                                "name": field.name,
                                "db_name": field.mapped_name(),
                                "type": field.ty,
                                "sql_type": sql_type,
                                "nullable": nullable,
                                "annotations": field.annotations
                                    .iter()
                                    .map(|ann| json!({
                                        "name": ann.name,
                                        "args": ann.args,
                                    }))
                                    .collect::<Vec<_>>(),
                            })
                        })
                        .collect::<Vec<_>>(),
                })
            })
            .collect::<Vec<_>>(),
    });
    serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
}

fn postgres_connection_string() -> String {
    let host = std::env::var("DB_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("DB_PORT").unwrap_or_else(|_| "55432".to_string());
    let name = std::env::var("DB_NAME").unwrap_or_else(|_| "linkhash_registry".to_string());
    let user = std::env::var("DB_USER").unwrap_or_else(|_| "postgres".to_string());
    let pass = std::env::var("DB_PASSWORD").unwrap_or_else(|_| "postgres".to_string());
    format!(
        "host={} port={} dbname={} user={} password={}",
        host, port, name, user, pass
    )
}

fn collect_migration_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    let entries =
        fs::read_dir(dir).map_err(|e| format!("failed to list {}: {}", dir.display(), e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("failed to read migration entry: {}", e))?;
        let path = entry.path();
        if path
            .extension()
            .and_then(|v| v.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("sql"))
            .unwrap_or(false)
        {
            files.push(path);
        }
    }
    Ok(files)
}

fn ensure_migrations_table(client: &mut Client) -> Result<(), postgres::Error> {
    client.batch_execute(
        "CREATE TABLE IF NOT EXISTS _deka_migrations (
            version TEXT PRIMARY KEY,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
        )",
    )
}

fn load_applied_migrations(
    client: &mut Client,
) -> Result<std::collections::HashSet<String>, postgres::Error> {
    let mut out = std::collections::HashSet::new();
    for row in client.query("SELECT version FROM _deka_migrations", &[])? {
        let version: String = row.get(0);
        out.insert(version);
    }
    Ok(out)
}

fn apply_migrations(
    client: &mut Client,
    migration_files: &[PathBuf],
    already_applied: &std::collections::HashSet<String>,
    log_scope: &str,
) -> Result<(usize, usize), String> {
    let mut applied_now = 0usize;
    let mut skipped = 0usize;
    for path in migration_files {
        let version = path
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("<unknown>")
            .to_string();
        if already_applied.contains(&version) {
            skipped += 1;
            continue;
        }
        let sql = fs::read_to_string(path)
            .map_err(|err| format!("failed to read {}: {}", path.display(), err))?;
        if sql.trim().is_empty() {
            skipped += 1;
            continue;
        }

        let mut tx = client
            .transaction()
            .map_err(|err| format!("failed to begin transaction: {}", err))?;
        if let Err(err) = tx.batch_execute(&sql) {
            let _ = tx.rollback();
            return Err(format!("migration {} failed: {}", version, err));
        }
        if let Err(err) = tx.execute(
            "INSERT INTO _deka_migrations (version) VALUES ($1)",
            &[&version],
        ) {
            let _ = tx.rollback();
            return Err(format!("failed to record migration {}: {}", version, err));
        }
        if let Err(err) = tx.commit() {
            return Err(format!("failed to commit migration {}: {}", version, err));
        }
        applied_now += 1;
        log(log_scope, &format!("applied {}", version));
    }
    Ok((applied_now, skipped))
}

fn render_init_migration(models: &[ModelDef]) -> String {
    let mut out = String::new();
    out.push_str("-- AUTO-GENERATED MIGRATION - DO NOT EDIT MANUALLY\n");
    out.push_str("-- Generated by deka db generate\n\n");
    for model in models {
        let table = to_table_name(&model.name);
        out.push_str(&format!("CREATE TABLE IF NOT EXISTS \"{}\" (\n", table));
        let mut defs: Vec<String> = Vec::new();
        let mut index_defs: Vec<String> = Vec::new();
        for field in &model.fields {
            let (sql_ty, nullable) = map_sql_type(&field.ty);
            let db_name = field.mapped_name();
            let mut def = if field.has_annotation("autoIncrement") {
                format!("  \"{}\" BIGSERIAL", db_name)
            } else {
                format!("  \"{}\" {}", db_name, sql_ty)
            };
            if !nullable {
                def.push_str(" NOT NULL");
            }
            if field.has_annotation("id") || field.name == "id" {
                def.push_str(" PRIMARY KEY");
            }
            if field.has_annotation("unique") {
                def.push_str(" UNIQUE");
            }
            if let Some(default_ann) = field.annotation("default") {
                if let Some(raw) = default_ann.args.first() {
                    let literal = default_sql_literal(raw);
                    def.push_str(&format!(" DEFAULT {}", literal));
                }
            }
            defs.push(def);

            if let Some(index_ann) = field.annotation("index") {
                let explicit = index_ann.args.first().map(|arg| unquote(arg));
                let index_name = explicit
                    .filter(|name| !name.is_empty())
                    .unwrap_or_else(|| format!("idx_{}_{}", table, db_name));
                index_defs.push(format!(
                    "CREATE INDEX IF NOT EXISTS \"{}\" ON \"{}\" (\"{}\");",
                    index_name, table, db_name
                ));
            }
        }
        out.push_str(&defs.join(",\n"));
        out.push_str("\n);\n\n");
        for idx in index_defs {
            out.push_str(&idx);
            out.push('\n');
        }
        if !model.fields.is_empty() {
            out.push('\n');
        }
    }
    out
}

fn default_sql_literal(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.eq_ignore_ascii_case("true") {
        "TRUE".to_string()
    } else if trimmed.eq_ignore_ascii_case("false") {
        "FALSE".to_string()
    } else if trimmed.eq_ignore_ascii_case("null") {
        "NULL".to_string()
    } else if looks_like_number(trimmed) {
        trimmed.to_string()
    } else {
        format!("'{}'", unquote(trimmed).replace('\'', "''"))
    }
}

fn looks_like_number(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    let mut start = 0usize;
    if bytes[0] == b'-' || bytes[0] == b'+' {
        start = 1;
    }
    if start >= bytes.len() {
        return false;
    }
    let mut saw_digit = false;
    let mut saw_dot = false;
    for &b in &bytes[start..] {
        if b.is_ascii_digit() {
            saw_digit = true;
            continue;
        }
        if b == b'.' && !saw_dot {
            saw_dot = true;
            continue;
        }
        return false;
    }
    saw_digit
}

fn unquote(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.len() >= 2 {
        let bytes = trimmed.as_bytes();
        if (bytes[0] == b'"' && bytes[trimmed.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[trimmed.len() - 1] == b'\'')
        {
            return trimmed[1..trimmed.len() - 1].to_string();
        }
    }
    trimmed.to_string()
}

fn to_table_name(model_name: &str) -> String {
    let mut out = String::new();
    for (idx, ch) in model_name.chars().enumerate() {
        if ch.is_uppercase() {
            if idx > 0 {
                out.push('_');
            }
            for lower in ch.to_lowercase() {
                out.push(lower);
            }
        } else {
            out.push(ch);
        }
    }
    if out.ends_with('s') {
        out
    } else {
        format!("{}s", out)
    }
}

fn map_sql_type(ty: &str) -> (&'static str, bool) {
    let trimmed = ty.trim();
    if let Some(inner) = trimmed
        .strip_prefix("Option<")
        .and_then(|s| s.strip_suffix('>'))
    {
        let (mapped, _) = map_sql_type(inner);
        return (mapped, true);
    }
    match trimmed {
        "int" | "i64" | "u64" | "i32" | "u32" => ("BIGINT", false),
        "float" | "double" | "f64" | "f32" => ("DOUBLE PRECISION", false),
        "bool" | "boolean" => ("BOOLEAN", false),
        "string" | "String" => ("TEXT", false),
        _ => ("TEXT", false),
    }
}

#[cfg(test)]
mod tests {
    use super::{extract_struct_models, resolve_generate_input, resolve_model_entry};
    use std::fs;
    use std::path::Path;

    #[test]
    fn rejects_missing_path() {
        let err = resolve_model_entry(Path::new("missing/thing.phpx"), "missing/thing.phpx")
            .expect_err("expected missing path to fail");
        assert!(err.contains("model input not found"));
    }

    #[test]
    fn resolves_project_alias_path() {
        let dir = tempfile::tempdir().expect("tempdir");
        let types_dir = dir.path().join("types");
        fs::create_dir_all(&types_dir).expect("create types");
        let model = types_dir.join("index.phpx");
        fs::write(&model, "struct User { $id: int @id }").expect("write model");

        let input = "@/types".to_string();
        let resolved = resolve_generate_input(dir.path(), Some(&input)).expect("resolve");
        assert_eq!(resolved, model);
    }

    #[test]
    fn extracts_models_and_fields() {
        let source = r#"
struct User {
  $id: Option<int> @id
  $email: string
}

struct Package {
  $name: string
}
"#;
        let models = extract_struct_models(source, "inline.phpx".to_string()).expect("models");
        assert_eq!(models.len(), 2);
        assert_eq!(models[0].name, "User");
        assert_eq!(models[0].fields.len(), 2);
        assert_eq!(models[0].fields[0].name, "id");
        assert_eq!(models[0].fields[0].ty, "Option<int>");
        assert_eq!(models[0].fields[0].annotations.len(), 1);
        assert_eq!(models[0].fields[0].annotations[0].name, "id");
        assert_eq!(models[1].name, "Package");
        assert_eq!(models[1].fields.len(), 1);
    }

    #[test]
    fn migration_respects_field_annotations() {
        let source = r#"
struct User {
  $id: int @id @autoIncrement
  $email: string @unique @map("email_address")
  $age: Option<int> @default(18)
  $name: string @index("users_name_idx")
}
"#;
        let models = extract_struct_models(source, "inline.phpx".to_string()).expect("models");
        let migration = super::render_init_migration(&models);
        assert!(migration.contains("\"id\" BIGSERIAL NOT NULL PRIMARY KEY"));
        assert!(migration.contains("\"email_address\" TEXT NOT NULL UNIQUE"));
        assert!(migration.contains("\"age\" BIGINT DEFAULT 18"));
        assert!(migration.contains("CREATE INDEX IF NOT EXISTS \"users_name_idx\""));
    }

    #[test]
    fn generated_client_has_query_builder_api() {
        let source = r#"
struct User {
  $id: int @id @autoIncrement
  $email: string @unique
}
"#;
        let models = extract_struct_models(source, "inline.phpx".to_string()).expect("models");
        let client = super::render_client_phpx(&models);
        assert!(client.contains("export function createClient"));
        assert!(client.contains("insert: function($model)"));
        assert!(client.contains("select: function()"));
        assert!(client.contains("update: function($model)"));
        assert!(client.contains("delete: function($model)"));
        assert!(client.contains("transaction: function($fn)"));
    }

    #[test]
    fn generated_schema_json_contains_db_names() {
        let source = r#"
struct User {
  $id: int @id @autoIncrement
  $email: string @map("email_address")
}
"#;
        let models = extract_struct_models(source, "inline.phpx".to_string()).expect("models");
        let schema = super::render_generated_schema_json(&models);
        assert!(schema.contains("\"table\": \"users\""));
        assert!(schema.contains("\"db_name\": \"email_address\""));
    }

    #[test]
    fn generate_db_artifacts_writes_expected_files() {
        let source = r#"
struct User {
  $id: int @id @autoIncrement
  $email: string @unique
}
"#;
        let models = extract_struct_models(source, "types/index.phpx".to_string()).expect("models");

        let dir = tempfile::tempdir().expect("tempdir");
        let source_path = dir.path().join("types").join("index.phpx");
        fs::create_dir_all(source_path.parent().expect("parent")).expect("mkdir");
        fs::write(&source_path, source).expect("write source");

        let generated =
            super::generate_db_artifacts(dir.path(), &source_path, &models).expect("generated");
        assert_eq!(generated, 6);
        assert!(dir.path().join("db/index.phpx").exists());
        assert!(dir.path().join("db/client.phpx").exists());
        assert!(dir.path().join("db/meta.phpx").exists());
        assert!(dir.path().join("db/_state.json").exists());
        assert!(dir.path().join("db/migrations/0001_init.sql").exists());
        assert!(dir.path().join("db/.generated/schema.json").exists());
    }

    #[test]
    fn maps_option_types_to_nullable_sql() {
        let (ty, nullable) = super::map_sql_type("Option<int>");
        assert_eq!(ty, "BIGINT");
        assert!(nullable);
    }

    #[test]
    fn table_name_is_snake_plural() {
        assert_eq!(super::to_table_name("User"), "users");
        assert_eq!(super::to_table_name("PackageVersion"), "package_versions");
    }
}
