use {
    crate::{command::Command, dirs::repository_path, output},
    std::path::PathBuf,
};

const SLOTS_PER_EPOCH: u64 = 100;

const SOLANA_PATH: &str = ".solana";

const SOLANA_CLI_PATH: &'static str = "target/debug/solana";
const SOLANA_TEST_VALIDATOR_CLI_PATH: &'static str = "target/debug/solana-test-validator";

const SOLANA_TEST_VALIDATOR_LEDGER_PATH: &'static str = "test-ledger";

const UPSTREAM_REPOSITORY: &str = "https://github.com/buffalojoec/solana.git";
const UPSTREAM_REPOSITORY_BRANCH: &str = "master"; // TODO

fn get_solana_path() -> PathBuf {
    repository_path().join(SOLANA_PATH)
}

fn get_solana_cargo_manifest_path() -> PathBuf {
    get_solana_path().join("Cargo.toml")
}

pub fn get_solana_cli_path() -> PathBuf {
    get_solana_path().join(SOLANA_CLI_PATH)
}

fn get_solana_test_validator_cli_path() -> PathBuf {
    get_solana_path().join(SOLANA_TEST_VALIDATOR_CLI_PATH)
}

fn get_solana_test_validator_ledger_path() -> PathBuf {
    get_solana_path().join(SOLANA_TEST_VALIDATOR_LEDGER_PATH)
}

fn fetch_changes() {
    let solana_path = get_solana_path();
    if solana_path.exists() {
        output::solana_fetching_latest_changes();
        let checkout_args = format!("checkout {}", UPSTREAM_REPOSITORY_BRANCH);
        Command::Git.command_with_dir(&checkout_args, &solana_path);
        Command::Git.command_with_dir("pull", &solana_path);
    } else {
        output::solana_cloning_repo();
        let args = format!(
            "clone {} --branch {} {}",
            UPSTREAM_REPOSITORY,
            UPSTREAM_REPOSITORY_BRANCH,
            solana_path.display()
        );
        Command::Git.command(&args);
    }
}

fn build_solana() {
    output::solana_building();
    let manifest_path = get_solana_cargo_manifest_path();
    let build_args = format!("build --manifest-path {}", manifest_path.display());
    Command::raw_command_with_dir("./cargo", &build_args, &get_solana_path())
}

fn start_test_validator() {
    output::starting_local_validator();
    let args = format!(
        "--slots-per-epoch {} --ledger {}",
        SLOTS_PER_EPOCH,
        get_solana_test_validator_ledger_path().display(),
    );
    Command::raw_command_detached_with_dir(
        get_solana_test_validator_cli_path().to_str().unwrap(),
        &args,
        &repository_path(),
    );
    // Give it a few seconds to boot up
    std::thread::sleep(std::time::Duration::from_secs(5));
}

fn delete_test_validator_ledger() {
    let ledger_path = get_solana_test_validator_ledger_path();
    if ledger_path.exists() {
        std::fs::remove_dir_all(ledger_path).unwrap();
    }
}

pub fn setup() {
    output::starting_setup();
    delete_test_validator_ledger();
    fetch_changes();
    build_solana();
    start_test_validator();
}

pub fn teardown() {
    delete_test_validator_ledger();
}
