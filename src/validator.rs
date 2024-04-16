use {
    solana_sdk::signature::Keypair,
    solana_test_validator::{TestValidator, TestValidatorGenesis},
};

pub async fn start() -> (TestValidator, Keypair) {
    solana_logger::setup();
    let test_validator_genesis = TestValidatorGenesis::default();
    test_validator_genesis.start_async().await
}
