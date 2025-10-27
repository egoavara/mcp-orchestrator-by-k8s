use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use wasm_bindgen_cli_support::Bindgen;

fn main() {
    println!("cargo:rerun-if-env-changed=SKIP_FRONTEND_BUILD");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let workspace_dir = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("backend crate should live inside workspace")
        .to_path_buf();
    let frontend_dir = workspace_dir.join("crates/mcp-orchestrator-front");

    println!(
        "cargo:rerun-if-changed={}",
        frontend_dir.join("src").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        frontend_dir.join("index.html").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        frontend_dir.join("styles.css").display()
    );

    if env::var("SKIP_FRONTEND_BUILD").is_ok() {
        return;
    }

    if env::var("TARGET")
        .map(|t| t.contains("wasm32"))
        .unwrap_or(false)
    {
        return;
    }

    build_frontend_wasm(&workspace_dir, &frontend_dir);
}

fn build_frontend_wasm(workspace_dir: &Path, frontend_dir: &Path) {
    if !ensure_wasm_target() {
        panic!(
            "wasm32-unknown-unknown 타깃을 설치할 수 없습니다. 'rustup target add wasm32-unknown-unknown'을 수동으로 실행하거나 SKIP_FRONTEND_BUILD=1 환경변수로 빌드를 건너뛰세요."
        );
    }

    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());

    // Use a separate target directory to avoid lock contention with parent cargo
    let frontend_target_dir = workspace_dir.join("target/frontend-wasm");

    let status = Command::new(&cargo)
        .current_dir(workspace_dir)
        .env("CARGO_TARGET_DIR", &frontend_target_dir)
        .args([
            "build",
            "--release",
            "-p",
            "mcp-orchestrator-front",
            "--lib",
            "--target",
            "wasm32-unknown-unknown",
        ])
        .status()
        .expect("failed to invoke cargo to build frontend");
    if !status.success() {
        panic!("frontend wasm 빌드가 실패했습니다. 'rustup target add wasm32-unknown-unknown'을 먼저 실행하세요.");
    }

    let wasm_path = frontend_target_dir.join("wasm32-unknown-unknown/release/mcp_orchestrator_front.wasm");
    if !wasm_path.exists() {
        panic!(
            "{} 경로에서 mcp_orchestrator_front.wasm을 찾을 수 없습니다.",
            wasm_path.display()
        );
    }

    let dist_dir = frontend_dir.join("dist");
    if dist_dir.exists() {
        fs::remove_dir_all(&dist_dir).expect("failed to clean previous dist directory");
    }
    fs::create_dir_all(&dist_dir).expect("failed to create dist directory");

    let mut bindgen = Bindgen::new();
    bindgen.input_path(&wasm_path);
    bindgen
        .web(true)
        .expect("wasm-bindgen web 출력 설정에 실패했습니다");
    bindgen
        .generate(&dist_dir)
        .expect("wasm-bindgen 변환에 실패했습니다");

    copy_and_fix_html(frontend_dir.join("index.html"), dist_dir.join("index.html"));
    copy_static(frontend_dir.join("styles.css"), dist_dir.join("styles.css"));
}

fn copy_and_fix_html(from: PathBuf, to: PathBuf) {
    let content = fs::read_to_string(&from)
        .unwrap_or_else(|err| panic!("Failed to read {}: {}", from.display(), err));
    
    let fixed_content = content
        .replace(r#"<link data-trunk rel="css" href="/statics/styles.css">"#, r#"<link rel="stylesheet" href="/statics/styles.css">"#);
    
    fs::write(&to, fixed_content)
        .unwrap_or_else(|err| panic!("Failed to write {}: {}", to.display(), err));
}

fn copy_static(from: PathBuf, to: PathBuf) {
    if let Err(err) = fs::copy(&from, &to) {
        panic!(
            "{} 파일을 {} 로 복사하지 못했습니다: {}",
            from.display(),
            to.display(),
            err
        );
    }
}

fn ensure_wasm_target() -> bool {
    if let Ok(output) = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout
            .lines()
            .any(|line| line.trim() == "wasm32-unknown-unknown")
        {
            return true;
        }
    }

    matches!(
        Command::new("rustup")
            .args(["target", "add", "wasm32-unknown-unknown"])
            .status(),
        Ok(status) if status.success()
    )
}
