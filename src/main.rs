//! CLI to test Core BPF program migrations.

use {
    clap::{Parser, Subcommand, ValueEnum},
    solana_sdk::pubkey::Pubkey,
};

#[derive(Clone, Debug, ValueEnum)]
enum Program {
    AddressLookupTable,
    Config,
    FeatureGate,
    Stake,
}
impl Program {
    pub fn id(&self) -> Pubkey {
        match self {
            Program::AddressLookupTable => solana_sdk::address_lookup_table::program::id(),
            Program::Config => solana_sdk::config::program::id(),
            Program::FeatureGate => solana_sdk::feature::id(),
            Program::Stake => solana_sdk::stake::program::id(),
        }
    }
}

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
        }
    }
    Ok(())
}
