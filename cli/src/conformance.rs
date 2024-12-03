//! Conformance testing handler.

use {
    crate::program::Program,
    solana_sdk::pubkey::Pubkey,
    std::{
        path::{Path, PathBuf},
        process::Command,
    },
};

const PATH_CONFORMANCE: &str = "./solana-conformance";
const PATH_PROGRAM_REPO: &str = "impl/program-repo";
const PATH_SF_AGAVE: &str = "impl/solfuzz-agave";
const PATH_TARGETS_DIR: &str = "impl/lib";
const PATH_TEST_VECTORS: &str = "impl/test-vectors";

/// Conformance testing handler.
pub struct ConformanceHandler {
    builtin_target_path: Option<PathBuf>,
    bpf_target_path: Option<PathBuf>,
    elf_path: PathBuf,
    fixtures_path: PathBuf,
    program_id: Pubkey,
}

impl ConformanceHandler {
    fn new(program: &Program, elf_directory: &str, fixtures_path: PathBuf) -> Self {
        Self {
            builtin_target_path: None,
            bpf_target_path: None,
            elf_path: Path::new(elf_directory).join(program.elf_name()),
            fixtures_path,
            program_id: program.program_id(),
        }
    }

    pub fn no_setup(program: &Program, elf_directory: &str, use_mollusk_fixtures: bool) -> Self {
        Self::new(
            program,
            elf_directory,
            if use_mollusk_fixtures {
                Path::new(PATH_CONFORMANCE)
                    .join(PATH_PROGRAM_REPO)
                    .join("program")
                    .join("fuzz")
                    .join("blob")
            } else {
                Path::new(PATH_CONFORMANCE)
                    .join(PATH_TEST_VECTORS)
                    .join(program.fixtures_path())
            },
        )
    }

    pub fn setup(program: &Program, elf_directory: &str, use_mollusk_fixtures: bool) -> Self {
        // Clone harnesses.
        git_clone(
            "https://github.com/firedancer-io/solana-conformance.git",
            "main",
            Path::new(PATH_CONFORMANCE),
        );
        git_clone(
            "http://github.com/buffalojoec/solfuzz-agave.git",
            "support-feature-gate-program",
            &Path::new(PATH_CONFORMANCE).join(PATH_SF_AGAVE),
        );

        // Set up fixtures.
        let fixtures_path = if use_mollusk_fixtures {
            // Use the Mollusk-generated fixtures from the program's repository.
            let path = Path::new(PATH_CONFORMANCE).join(PATH_PROGRAM_REPO);

            git_clone(&program.repository(), "main", &path);

            path.join("program").join("fuzz").join("blob")
        } else {
            // Use the fixtures provided by Firedancer.
            let path = Path::new(PATH_CONFORMANCE).join(PATH_TEST_VECTORS);

            git_clone(
                "https://github.com/firedancer-io/test-vectors.git",
                "main",
                &path,
            );

            let path = path.join(program.fixtures_path());

            // Remove skipped fixtures.
            for fix in program.skip_conformance_fixtures() {
                Command::new("rm")
                    .arg(path.join(fix).join(".fix"))
                    .status()
                    .expect("Failed to remove fixture");
            }

            path
        };

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

        Self::new(program, elf_directory, fixtures_path)
    }

    pub fn build_conformance_target_builtin(&mut self) {
        make_targets_dir();

        let manifest_path = Path::new(PATH_CONFORMANCE)
            .join(PATH_SF_AGAVE)
            .join("Cargo.toml");
        let src_path = Path::new(PATH_CONFORMANCE)
            .join(PATH_SF_AGAVE)
            .join("target/x86_64-unknown-linux-gnu/release/libsolfuzz_agave.so");
        let target_path = Path::new(PATH_CONFORMANCE)
            .join(PATH_TARGETS_DIR)
            .join("builtin.so");

        Command::new("cargo")
            .arg("build")
            .arg("--manifest-path")
            .arg(&manifest_path)
            .arg("--lib")
            .arg("--release")
            .arg("--target")
            .arg("x86_64-unknown-linux-gnu")
            .status()
            .expect("Failed to build target");

        mv(&src_path, &target_path);

        self.builtin_target_path = Some(target_path);
    }

    pub fn build_conformance_target_bpf(&mut self, conformance_mode: bool) {
        make_targets_dir();

        let manifest_path = Path::new(PATH_CONFORMANCE)
            .join(PATH_SF_AGAVE)
            .join("Cargo.toml");
        let src_path = Path::new(PATH_CONFORMANCE)
            .join(PATH_SF_AGAVE)
            .join("target/x86_64-unknown-linux-gnu/release/libsolfuzz_agave.so");
        let target_path = Path::new(PATH_CONFORMANCE)
            .join(PATH_TARGETS_DIR)
            .join("bpf.so");

        std::env::set_var("CORE_BPF_PROGRAM_ID", self.program_id.to_string());
        std::env::set_var("CORE_BPF_TARGET", cwd().join(&self.elf_path));
        std::env::set_var("FORCE_RECOMPILE", "true");

        let feature_flag = if conformance_mode {
            "core-bpf-conformance"
        } else {
            "core-bpf"
        };

        Command::new("cargo")
            .arg("build")
            .arg("--manifest-path")
            .arg(&manifest_path)
            .arg("--lib")
            .arg("--release")
            .arg("--target")
            .arg("x86_64-unknown-linux-gnu")
            .arg("--features")
            .arg(feature_flag)
            .status()
            .expect("Failed to build target");

        mv(&src_path, &target_path);

        std::env::remove_var("CORE_BPF_PROGRAM_ID");
        std::env::remove_var("CORE_BPF_TARGET");
        std::env::remove_var("FORCE_RECOMPILE");

        self.bpf_target_path = Some(target_path);
    }

    pub fn run_fixtures(&self) {
        let i_path = cwd().join(&self.fixtures_path);
        let t_path = cwd().join(
            self.bpf_target_path
                .as_ref()
                .expect("BPF target was not built"),
        );

        let output = Command::new("bash")
        .arg("-c")
        .arg(format!(
            "cd {} && source test_suite_env/bin/activate && solana-test-suite exec-fixtures -i {} -t {}",
            PATH_CONFORMANCE,
            i_path.display(),
            t_path.display()
        ))
        .output()
        .expect("Failed to run fixtures tests");

        let output = core::str::from_utf8(&output.stdout).unwrap();
        println!("{}", output);

        if output.contains("Failed tests:") {
            panic!("Test failed! Oh no!");
        }
    }

    pub fn run_conformance(&self) {
        let i_path = cwd().join(&self.fixtures_path);
        let s_path = cwd().join(
            self.builtin_target_path
                .as_ref()
                .expect("Builtin target was not built"),
        );
        let t_path = cwd().join(
            self.bpf_target_path
                .as_ref()
                .expect("BPF target was not built"),
        );

        Command::new("bash")
        .arg("-c")
        .arg(format!(
            "cd {} && source test_suite_env/bin/activate && solana-test-suite run-tests -i {} -s {} -t {}",
            PATH_CONFORMANCE,
            i_path.display(),
            s_path.display(),
            t_path.display()
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
}

fn cwd() -> PathBuf {
    std::env::current_dir().expect("Failed to get current working directory")
}

fn git_clone(url: &str, branch: &str, out_dir: &Path) {
    if std::fs::metadata(out_dir).is_ok() {
        Command::new("git")
            .arg("-C")
            .arg(out_dir)
            .arg("pull")
            .status()
            .expect("Failed to pull repo");
    } else {
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

fn make_targets_dir() {
    Command::new("mkdir")
        .current_dir(PATH_CONFORMANCE)
        .arg("-p")
        .arg(PATH_TARGETS_DIR)
        .status()
        .expect("Failed to create directory");
}

fn mv(src: &Path, dest: &Path) {
    Command::new("mv")
        .arg(src)
        .arg(dest)
        .status()
        .expect("Failed to move file");
}
