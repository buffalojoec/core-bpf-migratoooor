use {
    crate::output::progress_bar,
    solana_client::nonblocking::rpc_client::RpcClient,
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

const SLOTS_PER_EPOCH: u64 = 50;

pub struct ValidatorContext {
    pub test_validator: TestValidator,
    pub payer: Keypair,
}

impl ValidatorContext {
    pub fn rpc_client(&self) -> RpcClient {
        self.test_validator.get_async_rpc_client()
    }

    pub async fn wait_for_next_epoch(&self) {
        println!();
        let progress_bar = progress_bar();
        let rpc_client = self.rpc_client();

        let start_slot = rpc_client.get_slot().await.unwrap();
        let mut remaining_slots = SLOTS_PER_EPOCH - (start_slot % SLOTS_PER_EPOCH);

        progress_bar.set_message("Waiting for next epoch...");

        while remaining_slots > 1 {
            let this_slot = rpc_client.get_slot().await.unwrap();
            remaining_slots = SLOTS_PER_EPOCH - (this_slot % SLOTS_PER_EPOCH);
            progress_bar.inc(2);
            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        let epoch = rpc_client.get_epoch_info().await.unwrap().epoch;
        progress_bar.finish_with_message(format!("Epoch: {}", epoch));
        println!();
    }

    pub async fn start(feature_id: Pubkey, source_program_id: Pubkey, elf_path: PathBuf) -> Self {
        solana_logger::setup();

        let epoch_schedule = EpochSchedule::custom(SLOTS_PER_EPOCH, SLOTS_PER_EPOCH, false);
        let accounts = [(feature_id, feature::create_account(&Feature::default(), 42))];
        let programs = &[UpgradeableProgramInfo {
            program_id: source_program_id,
            loader: bpf_loader_upgradeable::id(),
            program_path: elf_path,
            upgrade_authority: Pubkey::new_unique(),
        }];

        let (test_validator, payer) = TestValidatorGenesis::default()
            .epoch_schedule(epoch_schedule)
            .add_accounts(accounts)
            .add_upgradeable_programs_with_path(programs)
            .start_async()
            .await;

        Self {
            test_validator,
            payer,
        }
    }
}
