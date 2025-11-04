use std::{env, path::PathBuf, process::Command};

fn main() {
    ensure_wasm_target();

    let workspace_dir = PathBuf::from(env::var("CARGO_WORKSPACE_DIR").expect("CARGO_WORKSPACE_DIR not set"));
    let frontend_dir = workspace_dir.join("crates/mcp-orchestrator-front");
    
    println!("cargo:rerun-if-changed={}", frontend_dir.display());

    // PROFILE 환경 변수로 release/debug 빌드 구분
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let is_release = profile == "release";
    
    // 별도의 target 디렉토리를 사용하여 Cargo 락 충돌 방지
    let frontend_target = workspace_dir.join("target/frontend-wasm");
    
    let mut trunk_cmd = Command::new("trunk");
    trunk_cmd
        .arg("build")
        .current_dir(&frontend_dir)
        // CARGO_TARGET_DIR을 별도로 설정하여 락 충돌 방지
        .env("CARGO_TARGET_DIR", &frontend_target);
    
    // release 빌드일 때만 --release 플래그 추가
    if is_release {
        trunk_cmd.arg("--release");
    }
    
    let status = trunk_cmd
        .status()
        .expect("Failed to execute trunk build command");

    if !status.success() {
        panic!("Trunk build failed : status {}", status);
    }
}

fn ensure_wasm_target() {
    // 설치된 타겟 확인
    let output = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .expect("Failed to execute rustup target list command");
    
    if !output.status.success() {
        eprintln!("Warning: Failed to list installed targets");
        return;
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.lines().any(|line| line.trim() == "wasm32-unknown-unknown") {
        // 이미 설치되어 있음
        return;
    }

    // 타겟 추가 시도
    println!("cargo:warning=wasm32-unknown-unknown target not found, attempting to install...");
    let status = Command::new("rustup")
        .args(["target", "add", "wasm32-unknown-unknown"])
        .status()
        .expect("Failed to execute rustup target add command");
    
    if !status.success() {
        eprintln!("Warning: Failed to add wasm32-unknown-unknown target");
        eprintln!("Please manually run: rustup target add wasm32-unknown-unknown");
    }
}
