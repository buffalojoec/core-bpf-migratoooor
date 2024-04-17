//! CLI to test Core BPF program migrations.

mod output;
mod program;
mod test_suite;
mod validator;

use {
    clap::{Parser, Subcommand},
    output::{output, title},
    program::Program,
    solana_sdk::bpf_loader_upgradeable,
    test_suite::TestContext,
    validator::ValidatorContext,
};

#[derive(Subcommand)]
enum SubCommand {
    /// Test the migration of a builtin program to Core BPF.
    Test {
        /// The program to migrate.
        program: Program,
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
        SubCommand::Test { program } => {
            let program_id = program.id();
            let feature_id = program.feature_id();
            let source_program_id = program.source_program_id();
            let elf_path = program.elf_path();

            title(&program_id);

            output("Starting test validator...");
            let validator = ValidatorContext::start(feature_id, source_program_id, elf_path).await;
            let ValidatorContext {
                payer,
                test_validator,
            } = &validator;
            let rpc_client = test_validator.get_async_rpc_client();
            let test_context = TestContext::new(&rpc_client, payer).await;

            output("Running tests on the builtin...");
            program.test(&test_context).await;

            validator.wait_for_next_epoch().await;

            assert!(rpc_client
                .get_account(&source_program_id)
                .await
                .ok()
                .is_none());
            assert!(rpc_client
                .get_account(&program_id)
                .await
                .ok()
                .map(|a| a.owner == bpf_loader_upgradeable::id())
                .unwrap_or(false));
            validator.wait_for_next_slot().await;
            output("Migration successful.");

            output("Running tests on the BPF version...");
            program.test(&test_context).await;

            validator.wait_for_next_epoch().await;

            output("Running tests (again) on the BPF version...");
            program.test(&test_context).await;

            output("Test complete!");
        }
    }
    Ok(())
}
