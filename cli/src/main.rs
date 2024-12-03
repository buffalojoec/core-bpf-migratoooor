//! CLI to test Core BPF program migration on feature activations.

mod client;
mod cluster;
mod conformance;
mod file;
mod output;
mod program;
mod validator;

use {
    crate::{
        client::CbmRpcClient,
        cluster::Cluster,
        conformance::ConformanceHandler,
        output::{output, title_conformance_test, title_fixtures_test, title_stub_test},
        program::Program,
        validator::{MigrationTarget, ValidatorContext},
    },
    clap::{Parser, Subcommand},
    std::{fs::File, io::Write, path::Path, process::Command},
};

const ELF_DIRECTORY: &str = "elfs";
const MANIFEST_PATH_ACTIVATOR: &str = "./programs/activator/Cargo.toml";
const MANIFEST_PATH_STUB: &str = "./programs/stub/Cargo.toml";

#[derive(Subcommand)]
enum SubCommand {
    /// Test the migration of a builtin program to Core BPF using a stub
    /// program ELF.
    ///
    /// This command does not require as input a path to an ELF, but instead
    /// will use the ELF from the `stub` program.
    ///
    /// The stub program has a deterministic processor, so the test suite in
    /// the program's crate can be used to ensure the migration was successful.
    Stub {
        /// The program to test.
        program: Program,
        /// Slots per epoch (defaults to 50).
        #[arg(short, long, default_value = "50")]
        slots_per_epoch: u64,
    },
    /// Test a buffer account's ELF against a suite of Firedancer fixtures.
    ///
    /// Clones the ELF from the buffer account and runs the fixtures against
    /// the original builtin.
    Fixtures {
        /// The program to test.
        program: Program,
        /// The cluster to clone the buffer account data from.
        #[arg(short, long, default_value = "mainnet-beta")]
        cluster: Cluster,
        /// Whether or not to use Mollusk fixtures. Uses Firedancer instead.
        #[arg(short, long, default_value = "false")]
        use_mollusk_fixtures: bool,
        /// Whether or not to skip installing the Firedancer tool suite.
        #[arg(short, long, default_value = "false")]
        skip_setup: bool,
    },
    /// Test a buffer account's ELF against the original builtin using
    /// Firedancer's conformance tooling.
    ///
    /// Clones the ELF from the buffer account and runs the conformance tests
    /// against the original builtin.
    Conformance {
        /// The program to test.
        program: Program,
        /// The cluster to clone the buffer account data from.
        #[arg(short, long, default_value = "mainnet-beta")]
        cluster: Cluster,
        /// Whether or not to use Mollusk fixtures. Uses Firedancer instead.
        #[arg(short, long, default_value = "false")]
        use_mollusk_fixtures: bool,
        /// Whether or not to skip installing the Firedancer tool suite.
        #[arg(short, long, default_value = "false")]
        skip_setup: bool,
    },
}

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    pub command: SubCommand,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match Cli::parse().command {
        SubCommand::Stub {
            program,
            slots_per_epoch,
        } => {
            let program_id = program.program_id();
            let feature_id = program.feature_gate();
            let buffer_address = program.buffer_address();

            title_stub_test(&feature_id, &buffer_address);

            output("Bulding programs...");
            cargo_build_sbf(MANIFEST_PATH_ACTIVATOR);
            cargo_build_sbf(MANIFEST_PATH_STUB);

            output("Starting test validator...");
            let context = ValidatorContext::start(
                &[MigrationTarget {
                    feature_id,
                    buffer_address,
                    elf_name: "cbm_program_stub",
                }],
                ELF_DIRECTORY,
                slots_per_epoch,
            )
            .await;

            output("Checking to see if program is currently a builtin...");
            if program_id != solana_sdk::feature::id() {
                context.assert_program_is_builtin(&program_id).await;
            }
            output("It is.");

            output(&format!("Activating feature {}...", feature_id));
            context.activate_feature(&feature_id).await;

            context.wait_for_next_epoch().await;

            output("Checking to see if program is now a BPF program...");
            context.assert_program_is_bpf(&program_id).await;
            output("It is.");

            context.wait_for_next_slot().await;

            output("Running stub tests on the BPF program...");
            context.run_stub_tests(&program_id).await;
            output("Success.");

            context.wait_for_next_epoch().await;

            output("Running stub tests again on the BPF program...");
            context.run_stub_tests(&program_id).await;
            output("Success.");

            output("Test complete! Woohoo!");
        }
        SubCommand::Fixtures {
            program,
            cluster,
            use_mollusk_fixtures,
            skip_setup,
        } => {
            let buffer_address = program.buffer_address();

            let cluster_string = cluster.to_string();
            let cluster_rpc_client = CbmRpcClient::new(cluster.url());

            title_fixtures_test(&cluster_string, &buffer_address, use_mollusk_fixtures);

            output(&format!("Cloning ELF from {}...", &cluster_string));
            let elf = cluster_rpc_client
                .clone_elf_from_buffer_account(&buffer_address)
                .await;

            output("Initializing test environment...");
            let mut handler = if skip_setup {
                ConformanceHandler::no_setup(&program, ELF_DIRECTORY, use_mollusk_fixtures)
            } else {
                ConformanceHandler::setup(&program, ELF_DIRECTORY, use_mollusk_fixtures)
            };

            output("Bulding target...");
            write_elf_to_file(elf, &program.elf_name());
            handler.build_conformance_target_bpf(/* conformance_mode */ false);

            output("Running fixtures...");
            handler.run_fixtures();

            output("Test complete! Woohoo!");
        }
        SubCommand::Conformance {
            program,
            cluster,
            use_mollusk_fixtures,
            skip_setup,
        } => {
            let buffer_address = program.buffer_address();

            let cluster_string = cluster.to_string();
            let cluster_rpc_client = CbmRpcClient::new(cluster.url());

            title_conformance_test(&cluster_string, &buffer_address, use_mollusk_fixtures);

            output(&format!("Cloning ELF from {}...", &cluster_string));
            let elf = cluster_rpc_client
                .clone_elf_from_buffer_account(&buffer_address)
                .await;

            output("Initializing test environment...");
            let mut handler = if skip_setup {
                ConformanceHandler::no_setup(&program, ELF_DIRECTORY, use_mollusk_fixtures)
            } else {
                ConformanceHandler::setup(&program, ELF_DIRECTORY, use_mollusk_fixtures)
            };

            output("Bulding targets...");
            write_elf_to_file(elf, &program.elf_name());
            handler.build_conformance_target_builtin();
            handler.build_conformance_target_bpf(/* conformance_mode */ true);

            output("Running conformance tests...");
            handler.run_conformance();

            output("Test complete! Woohoo!");
        }
    }

    Ok(())
}

fn cargo_build_sbf(manifest_path: &str) {
    Command::new("cargo")
        .arg("build-sbf")
        .arg("--manifest-path")
        .arg(manifest_path)
        .arg("--features")
        .arg("sbf-entrypoint")
        .arg("--sbf-out-dir")
        .arg(ELF_DIRECTORY)
        .status()
        .expect("Failed to build crate");
}

fn write_elf_to_file(elf: Vec<u8>, elf_name: &str) {
    std::fs::create_dir_all(ELF_DIRECTORY).unwrap();
    let path = Path::new(ELF_DIRECTORY).join(elf_name);
    let mut file = File::create(path).expect("Failed to create ELF file");
    file.write_all(&elf).expect("Failed to write ELF to file");
}
