//! Simple testing CLI for Programify Feature Gate

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
    solana_sdk::feature::ID as FEATURE_GATE_PROGRAM_ID,
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

            // Hit the program with a transaction
            client.expect_success(&program_id).await?;

            // Hit Feature Gate with a transaction
            client
                .expect_failure_program_missing(&FEATURE_GATE_PROGRAM_ID)
                .await?;

            // Activate the feature
            feature::activate_programify_feature_gate(&client).await?;

            // Hit Feature Gate with a transaction
            client.expect_success(&FEATURE_GATE_PROGRAM_ID).await?;

            // Hit the program with a transaction
            client.expect_failure_program_missing(&program_id).await?;

            // Wait another epoch
            client.poll_for_next_epoch().await?;
            client.poll_slots(5).await?;

            // Hit Feature Gate with a transaction
            client.expect_success(&FEATURE_GATE_PROGRAM_ID).await?;

            // Hit the program with a transaction
            client.expect_failure_program_missing(&program_id).await?;

            setup::teardown();

            output::test_concluded();
        }
    }
    Ok(())
}
