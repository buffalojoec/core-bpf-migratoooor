//! Simple testing CLI for Programify Address Lookup Table

mod client;
mod command;
mod dirs;
mod feature;
mod output;
mod program;
mod setup;

use {
    crate::client::Client,
    clap::{Parser, Subcommand},
    solana_sdk::address_lookup_table::program::ID as ADDRESS_LOOKUP_TABLE_PROGRAM_ID,
};

#[derive(Subcommand)]
enum SubCommand {
    Start {
        /// Enables the features surrounding deprecating the `executable` flag
        /// on accounts.
        /// See `https://github.com/solana-labs/solana/issues/33970`
        #[clap(short = 'e', long, action)]
        executable_features: bool,
    },
}

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    pub command: SubCommand,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        SubCommand::Start {
            executable_features,
        } => {
            output::start(executable_features);

            // Set up the Solana environment and start the local validator
            setup::setup(executable_features);

            // Build and deploy the program
            let program_id = program::build_and_deploy()?;

            let client = Client::new().await?;

            // Hit Address Lookup Table with a transaction
            client
                .expect_success(&ADDRESS_LOOKUP_TABLE_PROGRAM_ID)
                .await?;

            // Hit the program with a transaction
            client
                .expect_failure_invalid_instruction(&program_id)
                .await?;

            // Activate the feature
            feature::activate_migration_feature_gate(&client).await?;

            // Hit the program with a transaction
            client.expect_failure_program_missing(&program_id).await?;

            // Hit Address Lookup Table with a transaction
            client
                .expect_failure_invalid_instruction(&ADDRESS_LOOKUP_TABLE_PROGRAM_ID)
                .await?;

            setup::teardown();

            output::test_concluded();
        }
    }
    Ok(())
}
