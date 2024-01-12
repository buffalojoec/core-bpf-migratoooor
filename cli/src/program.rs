use {
    crate::{command::Command, dirs::repository_path, output},
    solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer},
    std::path::PathBuf,
};

const PROGRAM_NAME: &str = "feature_gate_noop";

fn get_program_so_path() -> PathBuf {
    repository_path()
        .join("program")
        .join("target")
        .join("deploy")
        .join(PROGRAM_NAME.to_owned() + ".so")
}

fn get_program_keypair_path() -> PathBuf {
    repository_path()
        .join("program")
        .join("target")
        .join("deploy")
        .join("feature_gate_noop-keypair.json")
}

fn get_cargo_manifest_path() -> PathBuf {
    repository_path().join("program").join("Cargo.toml")
}

/// Build the program
fn build() {
    let manifest_path = get_cargo_manifest_path();
    let build_args = format!("build-sbf --manifest-path {}", manifest_path.display());
    Command::cargo(&build_args);
}

/// Deploy the program
fn deploy() {
    let so_path = get_program_so_path();
    let deploy_args = format!("program deploy -ul {}", so_path.display());
    Command::solana(&deploy_args);
}

/// Read the program ID
fn read_program_id() -> Result<Pubkey, Box<dyn std::error::Error>> {
    let keypair_path = get_program_keypair_path();
    let file_contents = std::fs::read_to_string(keypair_path)?;
    let bytes: Vec<u8> = serde_json::from_str(&file_contents)?;
    let keypair = Keypair::from_bytes(&bytes)?;
    Ok(keypair.pubkey())
}

pub fn build_and_deploy() -> Result<Pubkey, Box<dyn std::error::Error>> {
    output::starting_build_and_deploy();
    build();
    deploy();
    read_program_id()
}
