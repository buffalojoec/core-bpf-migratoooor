use std::path::{Path, PathBuf};

const HOME_DIR: &str = "/Users/joesol";
const REPOSITORY_PATH: &str = "labs/programify-feature-gate";

pub fn home_dir() -> PathBuf {
    Path::new(HOME_DIR).to_owned()
}

pub fn repository_path() -> PathBuf {
    home_dir().join(REPOSITORY_PATH)
}
