//! Test validator with context for testing.

use {
    solana_sdk::{
        account::{AccountSharedData, WritableAccount},
        bpf_loader_upgradeable::{self, UpgradeableLoaderState},
        epoch_schedule::EpochSchedule,
        feature::Feature,
        pubkey::Pubkey,
        rent::Rent,
        signature::Keypair,
    },
    solana_test_validator::{TestValidator, TestValidatorGenesis},
};

pub struct MigrationTarget<'a> {
    feature_id: &'a Pubkey,
    buffer_address: &'a Pubkey,
    elf: &'a [u8],
}

pub struct ValidatorConfig {
    slots_per_epoch: u64,
}

pub struct ValidatorContext {
    pub test_validator: TestValidator,
    pub payer: Keypair,
}

impl ValidatorContext {
    pub async fn start(migration_targets: &[MigrationTarget<'_>], config: ValidatorConfig) -> Self {
        solana_logger::setup();

        let epoch_schedule = {
            let slots_per_epoch = config.slots_per_epoch;
            EpochSchedule::custom(slots_per_epoch, slots_per_epoch, false)
        };

        let accounts = migration_targets.iter().flat_map(|mt| {
            [
                (*mt.feature_id, staged_feature_account()),
                (*mt.buffer_address, buffer_account(mt.elf)),
            ]
        });

        let (test_validator, payer) = TestValidatorGenesis::default()
            .epoch_schedule(epoch_schedule)
            .add_accounts(accounts)
            .start_async()
            .await;

        Self {
            test_validator,
            payer,
        }
    }
}

// Create a "staged" feature account, owned by the activator program.
fn staged_feature_account() -> AccountSharedData {
    let space = Feature::size_of();
    let lamports = Rent::default().minimum_balance(space);
    AccountSharedData::new(lamports, space, &cbm_program_activator::id())
}

// Create a buffer account with the provided ELF.
fn buffer_account(elf: &[u8]) -> AccountSharedData {
    let space = UpgradeableLoaderState::size_of_buffer(elf.len());
    let lamports = Rent::default().minimum_balance(space);
    let mut account = AccountSharedData::new_data_with_space(
        lamports,
        &UpgradeableLoaderState::Buffer {
            authority_address: None,
        },
        space,
        &bpf_loader_upgradeable::id(),
    )
    .unwrap();
    account.data_as_mut_slice()[UpgradeableLoaderState::size_of_buffer_metadata()..]
        .copy_from_slice(elf);
    account
}
