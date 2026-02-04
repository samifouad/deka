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
    assert!(result.errors.iter().any(|err| err.kind == ErrorKind::ModuleError));
}

#[test]
fn module_missing_export_reports_error() {
    let path = fixtures_root().join("modules/missing_export.phpx");
    let result = compile_fixture(&path);
    if result
        .errors
        .iter()
        .any(|err| err.kind == ErrorKind::ModuleError)
    {
        return;
    }
    panic!("expected ModuleError, got: {:?}", result.errors);
}

#[test]
fn match_missing_case_reports_error() {
    let path = fixtures_root().join("patterns/enum_missing_case.phpx");
    let result = compile_fixture(&path);
    assert!(result.errors.iter().any(|err| err.kind == ErrorKind::PatternError));
}

#[test]
fn match_payload_mismatch_reports_error() {
    let path = fixtures_root().join("patterns/enum_payload_mismatch.phpx");
    let result = compile_fixture(&path);
    assert!(result.errors.iter().any(|err| err.kind == ErrorKind::PatternError));
}

#[test]
fn match_duplicate_case_reports_error() {
    let path = fixtures_root().join("patterns/enum_duplicate_case.phpx");
    let result = compile_fixture(&path);
    assert!(result.errors.iter().any(|err| err.kind == ErrorKind::PatternError));
}
