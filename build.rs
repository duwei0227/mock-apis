use std::path::Path;
use std::process::Command;

fn main() {
    let frontend = Path::new("frontend");

    println!("cargo:rerun-if-changed=frontend/src");
    println!("cargo:rerun-if-changed=frontend/package.json");
    println!("cargo:rerun-if-changed=frontend/vite.config.ts");

    if !frontend.join("node_modules").exists() {
        let status = Command::new("npm")
            .args(["install"])
            .current_dir(frontend)
            .status()
            .expect("failed to run npm install");
        assert!(status.success(), "npm install failed");
    }

    let status = Command::new("npm")
        .args(["run", "build"])
        .current_dir(frontend)
        .status()
        .expect("failed to run npm run build");
    assert!(status.success(), "npm run build failed");
}
