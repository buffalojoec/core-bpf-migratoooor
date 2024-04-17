//! CLI to test Core BPF program migrations.

mod harness;
mod output;
mod program;
mod validator;

use {
    clap::{Parser, Subcommand},
    output::{output, title},
    program::Program,
    solana_sdk::bpf_loader_upgradeable,
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
            let harness = program.harness();

            title(&program_id);

            output("Starting test validator...");
            let context = ValidatorContext::start(feature_id, source_program_id, elf_path).await;

            output("Running tests on the builtin...");
            harness.test(&context);

            context.wait_for_next_epoch().await;

            assert!(context.get_account(&source_program_id).await.is_none());
            assert!(context
                .get_account(&program_id)
                .await
                .map(|a| a.owner == bpf_loader_upgradeable::id())
                .unwrap_or(false));
            output("Migration successful.");

            output("Running tests on the BPF version...");
            harness.test(&context);

            context.wait_for_next_epoch().await;

            output("Running tests (again) on the BPF version...");
            harness.test(&context);

            output("Test complete!");
        }
    }
    Ok(())
}
