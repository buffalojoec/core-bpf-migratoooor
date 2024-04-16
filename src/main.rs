//! CLI to test Core BPF program migrations.

mod harness;
mod program;
mod validator;

use {
    clap::{Parser, Subcommand},
    program::Program,
    std::path::PathBuf,
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
            // Testing.
            println!("Testing migration of builtin programs to Core BPF");
            println!("Program: {}", program.id());
            let harness = program.harness();
            harness.test();

            // Core BPF migration stuff.
            use solana_runtime::bank::builtins::test_only::feature_gate_program;
            let feature_gate_feature_id = feature_gate_program::feature::id();
            let feature_gate_source_id = feature_gate_program::source_program::id();
            let feature_gate_source_path = PathBuf::from("./test_elf.so");

            let (test_validator, payer) = validator::start(
                feature_gate_feature_id,
                (feature_gate_source_id, feature_gate_source_path),
            )
            .await;
            {
                let local_validator = std::sync::Arc::new(test_validator);
                let _local_payer = std::sync::Arc::new(payer);
                let local_rpc_client = local_validator.get_async_rpc_client();

                let mut epoch = 0;
                while epoch < 2 {
                    let this_slot = local_rpc_client.get_slot().await.unwrap();
                    let this_epoch = local_rpc_client.get_epoch_info().await.unwrap().epoch;
                    println!("Slot: {} Epoch: {}", this_slot, this_epoch);
                    epoch = this_epoch;

                    // Sleep for 1 second.
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }

                // The source program should be gone.
                let source_program = local_rpc_client.get_account(&feature_gate_source_id).await;
                println!("Source program: {:?}", source_program);

                // We should have a program at the feature gate program ID.
                let feature_gate_program = local_rpc_client
                    .get_account(&solana_sdk::feature::id())
                    .await
                    .unwrap();
                println!("Feature gate program: {:?}", feature_gate_program);
            }
        }
    }
    Ok(())
}
