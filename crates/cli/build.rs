use std::process::Command;

fn main() {
    let git_sha = Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    let build_unix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());

    println!("cargo:rustc-env=DEKA_GIT_SHA={}", git_sha);
    println!("cargo:rustc-env=DEKA_BUILD_UNIX={}", build_unix);
    println!("cargo:rustc-env=DEKA_TARGET={}", target);
    println!("cargo:rustc-env=DEKA_RUNTIME_ABI=deka-runtime-phpx-v1");
}
