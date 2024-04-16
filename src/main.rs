//! CLI to test Core BPF program migrations.

mod harness;
mod program;
mod validator;

use {
    clap::{Parser, Subcommand},
    program::Program,
    solana_sdk::signer::Signer,
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
            println!("Testing migration of builtin programs to Core BPF");
            println!("Program: {}", program.id());
            let harness = program.harness();
            harness.test();
            let (test_validator, payer) = validator::start().await;
            {
                let local_validator = std::sync::Arc::new(test_validator);
                let local_payer = std::sync::Arc::new(payer);
                let local_rpc_client = local_validator.get_async_rpc_client();
                let bal = &local_rpc_client
                    .get_balance(&local_payer.pubkey())
                    .await
                    .unwrap();
                println!("Payer: {}, Balance: {}", local_payer.pubkey(), bal);
            }
        }
    }
    Ok(())
}
