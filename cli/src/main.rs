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
    Start,
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
        SubCommand::Start => {
            output::start();

            // Set up the Solana environment and start the local validator
            setup::setup();

            // Build and deploy the program
            let program_id = program::build_and_deploy()?;

            let client = Client::new().await?;

            // Hit Address Lookup Table with a transaction
            client
                .expect_success_no_return_data(&ADDRESS_LOOKUP_TABLE_PROGRAM_ID)
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
                .expect_success_with_return_data(&ADDRESS_LOOKUP_TABLE_PROGRAM_ID)
                .await?;

            // Wait another epoch
            output::warping_to_next_epoch();
            client.poll_for_next_epoch().await?;
            client.poll_slots(5).await?;

            // Hit the program with a transaction
            client.expect_failure_program_missing(&program_id).await?;

            // Hit Address Lookup Table with a transaction
            client
                .expect_success_with_return_data(&ADDRESS_LOOKUP_TABLE_PROGRAM_ID)
                .await?;

            // Send a versioned transaction using a lookup table
            client.expect_success_versioned_transaction().await?;

            setup::teardown();

            output::test_concluded();
        }
    }
    Ok(())
}
