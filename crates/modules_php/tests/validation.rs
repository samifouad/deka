use std::fs;
use std::path::{Path, PathBuf};

use bumpalo::Bump;

use modules_php::compiler_api::compile_phpx;
use modules_php::validation::{ErrorKind, ValidationResult};

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn load_fixture(path: &Path) -> String {
    fs::read_to_string(path).expect("fixture read failed")
}

fn compile_fixture(path: &Path) -> ValidationResult<'static> {
    let source = load_fixture(path);
    let arena = Box::leak(Box::new(Bump::new()));
    compile_phpx(&source, path.to_string_lossy().as_ref(), arena)
}

fn assert_has_error(result: &ValidationResult<'_>, kind: ErrorKind) {
    assert!(
        result.errors.iter().any(|err| err.kind == kind),
        "expected {:?}, got: {:?}",
        kind,
        result.errors
    );
}

#[test]
fn module_import_ok() {
    let path = fixtures_root().join("modules/basic.phpx");
    let result = compile_fixture(&path);
    assert!(result.errors.is_empty(), "unexpected errors: {:?}", result.errors);
}

#[test]
fn module_missing_module_reports_error() {
    let path = fixtures_root().join("modules/missing_module.phpx");
    let result = compile_fixture(&path);
    assert_has_error(&result, ErrorKind::ModuleError);
}

#[test]
fn module_missing_export_reports_error() {
    let path = fixtures_root().join("modules/missing_export.phpx");
    let result = compile_fixture(&path);
    assert_has_error(&result, ErrorKind::ModuleError);
}

#[test]
fn match_missing_case_reports_error() {
    let path = fixtures_root().join("patterns/enum_missing_case.phpx");
    let result = compile_fixture(&path);
    assert_has_error(&result, ErrorKind::PatternError);
}

#[test]
fn match_payload_mismatch_reports_error() {
    let path = fixtures_root().join("patterns/enum_payload_mismatch.phpx");
    let result = compile_fixture(&path);
    assert_has_error(&result, ErrorKind::PatternError);
}

#[test]
fn match_duplicate_case_reports_error() {
    let path = fixtures_root().join("patterns/enum_duplicate_case.phpx");
    let result = compile_fixture(&path);
    assert_has_error(&result, ErrorKind::PatternError);
}

#[test]
fn rule_null_reports_error() {
    let path = fixtures_root().join("rules/null_value.phpx");
    let result = compile_fixture(&path);
    assert_has_error(&result, ErrorKind::NullNotAllowed);
}

#[test]
fn rule_throw_reports_error() {
    let path = fixtures_root().join("rules/throw.phpx");
    let result = compile_fixture(&path);
    assert_has_error(&result, ErrorKind::ExceptionNotAllowed);
}

#[test]
fn rule_class_reports_error() {
    let path = fixtures_root().join("rules/class.phpx");
    let result = compile_fixture(&path);
    assert_has_error(&result, ErrorKind::OopNotAllowed);
}

#[test]
fn rule_namespace_reports_error() {
    let path = fixtures_root().join("rules/namespace.phpx");
    let result = compile_fixture(&path);
    assert_has_error(&result, ErrorKind::NamespaceNotAllowed);
}

#[test]
fn type_nullable_reports_error() {
    let path = fixtures_root().join("types/nullable_type.phpx");
    let result = compile_fixture(&path);
    assert_has_error(&result, ErrorKind::NullNotAllowed);
}

#[test]
fn type_union_reports_error() {
    let path = fixtures_root().join("types/unsupported_union.phpx");
    let result = compile_fixture(&path);
    assert_has_error(&result, ErrorKind::TypeError);
}

#[test]
fn type_generic_arity_reports_error() {
    let path = fixtures_root().join("types/generic_arity.phpx");
    let result = compile_fixture(&path);
    assert_has_error(&result, ErrorKind::TypeError);
}

#[test]
fn type_mismatch_reports_error() {
    let path = fixtures_root().join("types/type_mismatch.phpx");
    let result = compile_fixture(&path);
    assert_has_error(&result, ErrorKind::TypeError);
}

#[test]
fn struct_missing_field_reports_error() {
    let path = fixtures_root().join("structs/missing_field.phpx");
    let result = compile_fixture(&path);
    assert_has_error(&result, ErrorKind::StructError);
}

#[test]
fn struct_extra_field_reports_error() {
    let path = fixtures_root().join("structs/extra_field.phpx");
    let result = compile_fixture(&path);
    assert_has_error(&result, ErrorKind::StructError);
}

#[test]
fn jsx_unknown_component_reports_error() {
    let path = fixtures_root().join("jsx/unknown_component.phpx");
    let result = compile_fixture(&path);
    assert_has_error(&result, ErrorKind::JsxError);
}

#[test]
fn jsx_invalid_attr_reports_error() {
    let path = fixtures_root().join("jsx/invalid_attr.phpx");
    let result = compile_fixture(&path);
    assert_has_error(&result, ErrorKind::SyntaxError);
}

#[test]
fn frontmatter_missing_delimiter_reports_error() {
    let path = fixtures_root().join("frontmatter/missing_delimiter.phpx");
    let result = compile_fixture(&path);
    assert_has_error(&result, ErrorKind::JsxError);
}

#[test]
fn frontmatter_invalid_template_reports_error() {
    let path = fixtures_root().join("frontmatter/invalid_template.phpx");
    let result = compile_fixture(&path);
    assert_has_error(&result, ErrorKind::JsxError);
}

#[test]
fn multiple_errors_collected() {
    let path = fixtures_root().join("multiple_errors.phpx");
    let result = compile_fixture(&path);
    assert!(result.errors.len() >= 2, "expected multiple errors");
    assert_has_error(&result, ErrorKind::ModuleError);
    assert_has_error(&result, ErrorKind::NullNotAllowed);
}
