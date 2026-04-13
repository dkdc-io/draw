use std::path::Path;
use std::process::Command;

const WASM_FILE: &str = "frontend/draw_wasm_bg.wasm";
const JS_FILE: &str = "frontend/wasm_glue.js";

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_path = Path::new(&manifest_dir);
    let wasm_path = manifest_path.join(WASM_FILE);
    let js_path = manifest_path.join(JS_FILE);

    // Rerun if the artifacts are deleted or the wasm source changes
    println!("cargo:rerun-if-changed={}", wasm_path.display());
    println!("cargo:rerun-if-changed={}", js_path.display());
    println!("cargo:rerun-if-changed=../draw-wasm/src/");

    if wasm_path.exists() && js_path.exists() {
        return;
    }

    println!("cargo:warning=WASM artifacts missing, building with wasm-pack...");

    // Ensure the wasm32 target is installed (CI may not have it)
    let _ = Command::new("rustup")
        .args(["target", "add", "wasm32-unknown-unknown"])
        .status();

    let wasm_crate = manifest_path.join("../draw-wasm");
    let out_dir = manifest_path.join("../../target/wasm-pkg");

    let status = Command::new("wasm-pack")
        .args([
            "build",
            &wasm_crate.to_string_lossy(),
            "--target",
            "web",
            "--out-dir",
            &out_dir.to_string_lossy(),
        ])
        .status();

    match status {
        Ok(s) if s.success() => {}
        Ok(s) => {
            panic!(
                "wasm-pack build failed with exit code: {s}. \
                 Make sure wasm-pack is installed: cargo install wasm-pack"
            );
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            panic!("wasm-pack not found. Install it with: cargo install wasm-pack");
        }
        Err(e) => {
            panic!("Failed to run wasm-pack: {e}");
        }
    }

    // Copy artifacts to frontend/ (mirrors bin/build-wasm)
    let src_js = out_dir.join("draw_wasm.js");
    let src_wasm = out_dir.join("draw_wasm_bg.wasm");

    std::fs::copy(&src_js, &js_path).unwrap_or_else(|e| {
        panic!(
            "Failed to copy {} to {}: {e}",
            src_js.display(),
            js_path.display()
        )
    });
    std::fs::copy(&src_wasm, &wasm_path).unwrap_or_else(|e| {
        panic!(
            "Failed to copy {} to {}: {e}",
            src_wasm.display(),
            wasm_path.display()
        )
    });

    println!("cargo:warning=WASM artifacts built successfully");
}
