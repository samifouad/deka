use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR not set"));
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    let php_rs_path = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("unexpected manifest directory depth")
        .join("target/wasm32-unknown-unknown/release/php_rs.wasm");
    let src = std::env::var("PHP_WASM_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| php_rs_path);
    if !src.exists() {
        panic!(
            "PHP wasm binary not found at {}. Build php-rs wasm or run `cargo build -p php-rs --release --target wasm32-unknown-unknown --lib --no-default-features`, or set PHP_WASM_PATH",
            src.display()
        );
    }
    let dest = out_dir.join("php_rs.wasm");
    fs::copy(&src, &dest).expect("failed to copy php wasm binary");
    println!("cargo:rerun-if-changed={}", src.display());
    println!("cargo:rerun-if-env-changed=PHP_WASM_PATH");

    let protoc = protoc_bin_vendored::protoc_bin_path().expect("failed to locate protoc");
    // SAFETY: build scripts run single-threaded per crate; setting process env for prost-build is safe here.
    unsafe {
        std::env::set_var("PROTOC", protoc);
    }
    let proto_root = manifest_dir.join("proto");
    let db_proto = proto_root.join("bridge/v1/db.proto");
    let fs_proto = proto_root.join("bridge/v1/fs.proto");
    println!("cargo:rerun-if-changed={}", db_proto.display());
    println!("cargo:rerun-if-changed={}", fs_proto.display());
    let mut config = prost_build::Config::new();
    config.btree_map(["."]);
    config
        .compile_protos(&[db_proto, fs_proto], &[proto_root])
        .expect("failed to compile protobuf schemas");
}
