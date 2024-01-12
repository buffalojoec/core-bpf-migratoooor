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

            setup::teardown();

            output::test_concluded();
        }
    }
    Ok(())
}
