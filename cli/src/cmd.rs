//! OS commands, mostly cargo.

use {
    crate::program::Program,
    std::{
        path::{Path, PathBuf},
        process::Command,
    },
};

// -- Stub tests --

const MANIFEST_PATH_ACTIVATOR: &str = "./programs/activator/Cargo.toml";
const MANIFEST_PATH_STUB: &str = "./programs/stub/Cargo.toml";

fn cargo_build_sbf(manifest_path: &str, out_dir: &str) {
    Command::new("cargo")
        .arg("build-sbf")
        .arg("--manifest-path")
        .arg(manifest_path)
        .arg("--features")
        .arg("sbf-entrypoint")
        .arg("--sbf-out-dir")
        .arg(out_dir)
        .status()
        .expect("Failed to build crate");
}

pub fn build_programs(out_dir: &str) {
    cargo_build_sbf(MANIFEST_PATH_ACTIVATOR, out_dir);
    cargo_build_sbf(MANIFEST_PATH_STUB, out_dir);
}

// -- Conformance tests --

const PATH_CONFORMANCE: &str = "./solana-conformance";
const PATH_FIXTURES: &str = "impl/test-vectors";
const PATH_SF_AGAVE: &str = "impl/solfuzz-agave";
const PATH_TARGETS_DIR: &str = "impl/lib";

fn cwd() -> PathBuf {
    std::env::current_dir().expect("Failed to get current working directory")
}

fn git_clone(url: &str, branch: &str, out_dir: &str) {
    if std::fs::metadata(out_dir).is_ok() {
        Command::new("git")
            .arg("-C")
            .arg(out_dir)
            .arg("pull")
            .status()
            .expect("Failed to pull repo");
    } else {
        // If the directory doesn't exist, perform a git clone
        println!("Directory does not exist, performing git clone...");
        Command::new("git")
            .arg("clone")
            .arg("--branch")
            .arg(branch)
            .arg(url)
            .arg(out_dir)
            .status()
            .expect("Failed to clone repo");
    }
}

fn mv(src: &str, dest: &str) {
    Command::new("mv")
        .arg(src)
        .arg(dest)
        .status()
        .expect("Failed to move file");
}

pub fn build_conformance_test_environment(program: &Program) {
    // Clone necessary repositories.
    git_clone(
        "https://github.com/firedancer-io/solana-conformance.git",
        "main",
        PATH_CONFORMANCE,
    );
    git_clone(
        "https://github.com/firedancer-io/test-vectors.git",
        "main",
        &format!("{}/{}", PATH_CONFORMANCE, PATH_FIXTURES),
    );
    git_clone(
        "http://github.com/buffalojoec/solfuzz-agave.git",
        "core-bpf-conformance",
        &format!("{}/{}", PATH_CONFORMANCE, PATH_SF_AGAVE),
    );

    // Remove skipped fixtures.
    for fix in program.skip_conformance_fixtures() {
        Command::new("rm")
            .arg(format!(
                "{}/{}/{}/{}.fix",
                PATH_CONFORMANCE,
                PATH_FIXTURES,
                program.fixtures_path(),
                fix
            ))
            .status()
            .expect("Failed to remove fixture");
    }

    // Fetch protos.
    Command::new("make")
        .current_dir(PATH_CONFORMANCE)
        .arg("-j")
        .arg("-C")
        .arg(PATH_SF_AGAVE)
        .arg("fetch_proto")
        .status()
        .expect("Failed to fetch protobufs");

    // Build environment.
    Command::new("bash")
        .current_dir(PATH_CONFORMANCE)
        .arg("install_ubuntu_lite.sh")
        .status()
        .expect("Failed to install dependencies");
}

pub fn build_conformance_target_builtin() {
    // Make a directory for targets.
    Command::new("mkdir")
        .current_dir(PATH_CONFORMANCE)
        .arg("-p")
        .arg(PATH_TARGETS_DIR)
        .status()
        .expect("Failed to create directory");

    // Build the builtin target.
    Command::new("cargo")
        .arg("build")
        .arg("--manifest-path")
        .arg(format!("{}/{}/Cargo.toml", PATH_CONFORMANCE, PATH_SF_AGAVE))
        .arg("--lib")
        .arg("--release")
        .arg("--target")
        .arg("x86_64-unknown-linux-gnu")
        .status()
        .expect("Failed to build crate");
    mv(
        &format!(
            "{}/{}/target/x86_64-unknown-linux-gnu/release/libsolfuzz_agave.so",
            PATH_CONFORMANCE, PATH_SF_AGAVE,
        ),
        &format!("{}/{}/builtin.so", PATH_CONFORMANCE, PATH_TARGETS_DIR),
    );
}

pub fn build_conformance_target_bpf(program: &Program) {
    // Make a directory for targets.
    Command::new("mkdir")
        .current_dir(PATH_CONFORMANCE)
        .arg("-p")
        .arg(PATH_TARGETS_DIR)
        .status()
        .expect("Failed to create directory");

    std::env::set_var("CORE_BPF_PROGRAM_ID", program.program_id().to_string());
    std::env::set_var(
        "CORE_BPF_TARGET",
        format!("{}/elfs/{}", cwd().display(), program.elf_name()),
    );
    std::env::set_var("FORCE_RECOMPILE", "true");

    // Build the BPF target.
    Command::new("cargo")
        .arg("build")
        .arg("--manifest-path")
        .arg(format!("{}/{}/Cargo.toml", PATH_CONFORMANCE, PATH_SF_AGAVE))
        .arg("--lib")
        .arg("--release")
        .arg("--target")
        .arg("x86_64-unknown-linux-gnu")
        .arg("--features")
        .arg("core-bpf-conformance")
        .status()
        .expect("Failed to build crate");
    mv(
        &format!(
            "{}/{}/target/x86_64-unknown-linux-gnu/release/libsolfuzz_agave.so",
            PATH_CONFORMANCE, PATH_SF_AGAVE,
        ),
        &format!("{}/{}/core_bpf.so", PATH_CONFORMANCE, PATH_TARGETS_DIR),
    );

    std::env::remove_var("CORE_BPF_PROGRAM_ID");
    std::env::remove_var("CORE_BPF_TARGET");
    std::env::remove_var("FORCE_RECOMPILE");
}

pub fn run_fixtures(program: &Program) {
    let i_path = format!(
        "{}/{}/{}/{}",
        cwd().display(),
        PATH_CONFORMANCE,
        PATH_FIXTURES,
        program.fixtures_path()
    );
    let t_path = format!(
        "{}/{}/{}/core_bpf.so",
        cwd().display(),
        PATH_CONFORMANCE,
        PATH_TARGETS_DIR
    );

    let output = Command::new("bash")
        .arg("-c")
        .arg(format!(
            "cd {} && source test_suite_env/bin/activate && solana-test-suite exec-fixtures -i {} -t {}",
            PATH_CONFORMANCE,
            i_path,
            t_path
        ))
        .output()
        .expect("Failed to run fixtures tests");

    let output = core::str::from_utf8(&output.stdout).unwrap();
    println!("{}", output);

    if output.contains("Failed tests:") {
        panic!("Test failed! Oh no!");
    }
}

pub fn run_conformance(program: &Program) {
    let i_path = format!(
        "{}/{}/{}/{}",
        cwd().display(),
        PATH_CONFORMANCE,
        PATH_FIXTURES,
        program.fixtures_path()
    );
    let s_path = format!(
        "{}/{}/{}/builtin.so",
        cwd().display(),
        PATH_CONFORMANCE,
        PATH_TARGETS_DIR
    );
    let t_path = format!(
        "{}/{}/{}/core_bpf.so",
        cwd().display(),
        PATH_CONFORMANCE,
        PATH_TARGETS_DIR
    );

    Command::new("bash")
        .arg("-c")
        .arg(format!(
            "cd {} && source test_suite_env/bin/activate && solana-test-suite run-tests -i {} -s {} -t {}",
            PATH_CONFORMANCE,
            i_path,
            s_path,
            t_path
        ))
        .status()
        .expect("Failed to run conformance tests");

    let failed_protos_path = Path::new(PATH_CONFORMANCE).join("test_results/failed_protobufs");
    if failed_protos_path.is_dir()
        && !std::fs::read_dir(failed_protos_path)
            .map(|mut entries| entries.next().is_none())
            .unwrap_or(false)
    {
        panic!("Test failed! Oh no!");
    }
}
