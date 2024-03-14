use {
    crate::{
        command::Command,
        dirs::{read_pubkey_from_keypair_path, repository_path},
        output,
    },
    dotenv::dotenv,
    solana_sdk::pubkey::Pubkey,
    std::{env, path::PathBuf},
};

const PROGRAM_NAME: &str = "scbpf_address_lookup_table";

fn get_program_so_path() -> PathBuf {
    repository_path()
        .join("program")
        .join("target")
        .join("deploy")
        .join(PROGRAM_NAME.to_owned() + ".so")
}

fn get_program_keypair_path() -> String {
    dotenv().ok();
    env::var("PROGRAM_KEYPAIR_PATH").expect("PROGRAM_KEYPAIR_PATH variable")
}

fn get_cargo_manifest_path() -> PathBuf {
    repository_path().join("program").join("Cargo.toml")
}

/// Build the program
fn build() {
    let manifest_path = get_cargo_manifest_path();
    let build_args = format!(
        "build-sbf --manifest-path {} --features bpf-entrypoint",
        manifest_path.display()
    );
    Command::Cargo.command(&build_args);
}

/// Deploy the program
fn deploy() {
    let so_path = get_program_so_path();
    let deploy_args = format!(
        "program deploy -ul --program-id {} {}",
        get_program_keypair_path(),
        so_path.display(),
    );
    Command::Solana.command(&deploy_args);
}

/// Read the program ID
fn read_program_id() -> Result<Pubkey, Box<dyn std::error::Error>> {
    let keypair_path = get_program_keypair_path();
    read_pubkey_from_keypair_path(&keypair_path)
}

pub fn build_and_deploy() -> Result<Pubkey, Box<dyn std::error::Error>> {
    output::starting_build_and_deploy();
    build();
    deploy();
    read_program_id()
}
