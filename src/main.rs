//! CLI to test Core BPF program migrations.

mod harness;
mod program;

use {
    clap::{Parser, Subcommand},
    program::Program,
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
        }
    }
    Ok(())
}
