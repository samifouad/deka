// Minimal PHP runtime module - no heavy dependencies

use deno_core::op2;
use php_rs::parser::ast::{ClassKind, ClassMember, Program, Stmt, Type as AstType};
use php_rs::parser::lexer::Lexer;
use php_rs::parser::parser::{Parser, ParserMode};
use php_rs::parser::lexer::token::Token;
use bumpalo::Bump;
use std::collections::{HashMap, HashSet};
use wit_parser::{Resolve, Results, Type, TypeDefKind, TypeId, WorldItem, WorldKey};

/// Embedded PHP WASM binary produced by the `php-rs` crate.
static PHP_WASM_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/php_rs.wasm"));

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
    let mut wrapped_source = None;
    let source_bytes = if source.contains("<?php") {
        source.as_bytes()
    } else {
        let wrapped = format!("<?php\n{}", source);
        wrapped_source = Some(wrapped);
        wrapped_source.as_ref().unwrap().as_bytes()
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

#[op2]
#[serde]
fn op_php_read_env() -> HashMap<String, String> {
    std::env::vars().collect()
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
        op_php_read_env,
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
