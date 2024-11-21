//! OS commands, mostly cargo.

use std::process::Command;

const MANIFEST_PATH_ACTIVATOR: &str = "./programs/activator/Cargo.toml";
const MANIFEST_PATH_STUB: &str = "./programs/stub/Cargo.toml";

fn cargo_build_sbf(manifest_path: &str, out_dir: &str) {
    Command::new("cargo")
        .arg("build-sbf")
        .arg("--manifest-path")
        .arg(manifest_path)
        .arg("--sbf-out-dir")
        .arg(out_dir)
        .status()
        .expect("Failed to build crate");
}

pub fn build_programs(out_dir: &str) {
    cargo_build_sbf(MANIFEST_PATH_ACTIVATOR, out_dir);
    cargo_build_sbf(MANIFEST_PATH_STUB, out_dir);
}
