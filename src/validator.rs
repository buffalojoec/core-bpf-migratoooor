use {
    solana_sdk::{
        bpf_loader_upgradeable,
        epoch_schedule::EpochSchedule,
        feature::{self, Feature},
        pubkey::Pubkey,
        signature::Keypair,
    },
    solana_test_validator::{TestValidator, TestValidatorGenesis, UpgradeableProgramInfo},
    std::path::PathBuf,
};

pub async fn start(
    feature_id: Pubkey,
    source_program: (Pubkey, PathBuf),
) -> (TestValidator, Keypair) {
    solana_logger::setup();

    let (source_program_id, source_program_path) = source_program;

    let mut test_validator_genesis = TestValidatorGenesis::default();

    let slots_per_epoch = 50;
    test_validator_genesis.epoch_schedule(EpochSchedule::custom(
        slots_per_epoch,
        slots_per_epoch,
        /* enable_warmup_epochs = */ false,
    ));

    test_validator_genesis
        .add_accounts([(feature_id, feature::create_account(&Feature::default(), 42))]);

    test_validator_genesis.add_upgradeable_programs_with_path(&[UpgradeableProgramInfo {
        program_id: source_program_id,
        loader: bpf_loader_upgradeable::id(),
        program_path: source_program_path,
        upgrade_authority: Pubkey::new_unique(),
    }]);

    test_validator_genesis.start_async().await
}
