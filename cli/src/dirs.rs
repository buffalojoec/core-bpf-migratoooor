use {
    solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer},
    std::path::{Path, PathBuf},
};

const HOME_DIR: &str = "/Users/joesol";
const REPOSITORY_PATH: &str = "labs/programify-feature-gate";

pub fn home_dir() -> PathBuf {
    Path::new(HOME_DIR).to_owned()
}

pub fn repository_path() -> PathBuf {
    home_dir().join(REPOSITORY_PATH)
}

pub fn read_pubkey_from_keypair_path(path: &str) -> Result<Pubkey, Box<dyn std::error::Error>> {
    let file_contents = std::fs::read_to_string(path)?;
    let bytes: Vec<u8> = serde_json::from_str(&file_contents)?;
    let keypair = Keypair::from_bytes(&bytes)?;
    Ok(keypair.pubkey())
}
