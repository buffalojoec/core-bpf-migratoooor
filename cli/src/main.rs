//! CLI to test Core BPF program migration on feature activations.

mod cmd;
mod output;

use {
    crate::{
        cmd::build_programs,
        output::{output, title_stub_test},
    },
    cbm_harness::validator::{MigrationTarget, ValidatorContext},
    clap::{Parser, Subcommand},
    solana_sdk::pubkey::Pubkey,
    std::str::FromStr,
};

const ELF_DIRECTORY: &str = "elfs";

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
    StubTest {
        /// The program ID of the builtin.
        program_id: String,
        /// The feature ID for the migration.
        feature_id: String,
        /// The buffer address where the ELF should be stored.
        buffer_address: String,
        /// Slots per epoch (defaults to 50).
        #[arg(short, long, default_value = "50")]
        slots_per_epoch: u64,
    },
    // /// Test the migration of a builtin program to Core BPF using a custom
    // /// program ELF.
    // ///
    // /// (Coming soon)
    // LiveTest {
    //     /// The feature ID for the migration.
    //     feature_id: String,
    //     /// The buffer address where the ELF should be stored.
    //     buffer_address: String,
    //     /// The path to the ELF file.
    //     elf_path: String,
    //     /// Slots per epoch (defaults to 50).
    //     #[arg(short, long, default_value = "50")]
    //     slots_per_epoch: u64,
    // },
}

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    pub command: SubCommand,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match Cli::parse().command {
        SubCommand::StubTest {
            program_id,
            feature_id,
            buffer_address,
            slots_per_epoch,
        } => {
            let program_id = Pubkey::from_str(&program_id).expect("Invalid program ID");
            let feature_id = Pubkey::from_str(&feature_id).expect("Invalid feature ID");
            let buffer_address = Pubkey::from_str(&buffer_address).expect("Invalid buffer address");

            title_stub_test(&feature_id, &buffer_address);

            output("Bulding programs...");
            build_programs(ELF_DIRECTORY);

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
    }

    Ok(())
}
