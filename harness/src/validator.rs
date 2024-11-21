//! Test validator with context for testing.

use {
    crate::file::FileReader,
    indicatif::{ProgressBar, ProgressStyle},
    solana_rpc::rpc::JsonRpcConfig,
    solana_rpc_client_api::config::RpcTransactionConfig,
    solana_sdk::{
        account::{AccountSharedData, WritableAccount},
        bpf_loader_upgradeable::{self, UpgradeableLoaderState},
        commitment_config::CommitmentConfig,
        epoch_schedule::EpochSchedule,
        feature::Feature,
        instruction::Instruction,
        pubkey::Pubkey,
        rent::Rent,
        signature::{Keypair, Signature},
        signer::Signer,
        transaction::Transaction,
    },
    solana_test_validator::{TestValidator, TestValidatorGenesis, UpgradeableProgramInfo},
    solana_transaction_status::UiTransactionEncoding,
    std::path::PathBuf,
};

pub struct MigrationTarget<'a> {
    pub feature_id: Pubkey,
    pub buffer_address: Pubkey,
    pub elf_name: &'a str,
}

pub struct ValidatorContext {
    pub test_validator: TestValidator,
    pub payer: Keypair,
    pub slots_per_epoch: u64,
}

impl ValidatorContext {
    pub async fn get_account(&self, account_id: &Pubkey) -> solana_sdk::account::Account {
        self.test_validator
            .get_async_rpc_client()
            .get_account(account_id)
            .await
            .unwrap()
    }

    pub async fn assert_program_is_builtin(&self, program_id: &Pubkey) {
        let account = self.get_account(program_id).await;
        assert!(
            account.owner == solana_sdk::native_loader::id(),
            "Program is not a builtin"
        );
    }

    pub async fn assert_program_is_bpf(&self, program_id: &Pubkey) {
        let account = self.get_account(program_id).await;
        assert!(
            account.owner == solana_sdk::bpf_loader_upgradeable::id(),
            "Program is not BPF"
        );
    }

    pub async fn send_transaction(
        &self,
        instructions: &[Instruction],
        payer: &Pubkey,
        signers: &[&Keypair],
    ) -> Signature {
        let rpc_client = self.test_validator.get_async_rpc_client();
        let latest_blockhash = rpc_client.get_latest_blockhash().await.unwrap();
        let transaction = Transaction::new_signed_with_payer(
            instructions,
            Some(payer),
            signers,
            latest_blockhash,
        );
        rpc_client
            .send_and_confirm_transaction_with_spinner_and_commitment(
                &transaction,
                CommitmentConfig::confirmed(),
            )
            .await
            .unwrap()
    }

    pub async fn activate_feature(&self, feature_id: &Pubkey) {
        self.send_transaction(
            &[cbm_program_activator::activate_feature(feature_id)],
            &self.payer.pubkey(),
            &[&self.payer],
        )
        .await;
    }

    pub async fn run_stub_test_write(&self, program_id: &Pubkey) {
        let target = Keypair::new();
        let write_data = Pubkey::new_unique().to_bytes();
        self.send_transaction(
            &[cbm_program_stub::write(
                program_id,
                &target.pubkey(),
                &self.payer.pubkey(),
                &write_data,
            )],
            &self.payer.pubkey(),
            &[&self.payer, &target],
        )
        .await;

        let target_account = self.get_account(&target.pubkey()).await;

        assert_eq!(target_account.owner, *program_id);
        assert_eq!(target_account.data, write_data);
    }

    #[allow(deprecated)]
    pub async fn run_stub_test_emit(&self, program_id: &Pubkey) {
        let emit_data = Pubkey::new_unique().to_bytes();
        let signature = self
            .send_transaction(
                &[cbm_program_stub::emit(program_id, &emit_data)],
                &self.payer.pubkey(),
                &[&self.payer],
            )
            .await;

        let return_data = self
            .test_validator
            .get_async_rpc_client()
            .get_transaction_with_config(
                &signature,
                RpcTransactionConfig {
                    encoding: Some(UiTransactionEncoding::Base64),
                    commitment: Some(CommitmentConfig::confirmed()),
                    max_supported_transaction_version: None,
                },
            )
            .await
            .unwrap()
            .transaction
            .meta
            .unwrap()
            .return_data
            .unwrap();

        assert_eq!(return_data.program_id, program_id.to_string());
        assert_eq!(base64::decode(return_data.data.0).unwrap(), emit_data);
    }

    pub async fn run_stub_tests(&self, program_id: &Pubkey) {
        self.run_stub_test_write(program_id).await;
        // self.run_stub_test_emit(program_id).await;
    }

    pub async fn wait_for_next_slot(&self) {
        let rpc_client = self.test_validator.get_async_rpc_client();
        let start_slot = rpc_client.get_slot().await.unwrap();
        let mut slot = start_slot;

        while slot == start_slot {
            slot = rpc_client.get_slot().await.unwrap();
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
    }

    pub async fn wait_for_next_epoch(&self) {
        println!();
        let progress_bar = progress_bar();
        let rpc_client = self.test_validator.get_async_rpc_client();

        let slots_per_epoch = self.slots_per_epoch;
        let get_remaining_slots = |this_slot: u64| slots_per_epoch - (this_slot % slots_per_epoch);

        let start_slot = rpc_client.get_slot().await.unwrap();
        let mut remaining_slots = get_remaining_slots(start_slot);

        progress_bar.set_message("Waiting for next epoch...");

        while remaining_slots > 1 {
            let this_slot = rpc_client.get_slot().await.unwrap();
            remaining_slots = get_remaining_slots(this_slot);
            progress_bar.inc(2);
            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        let epoch = rpc_client.get_epoch_info().await.unwrap().epoch;
        progress_bar.finish_with_message(format!("Epoch: {}", epoch));
        println!();
    }

    pub async fn start(
        migration_targets: &[MigrationTarget<'_>],
        elf_directory: &str,
        slots_per_epoch: u64,
    ) -> Self {
        solana_logger::setup();

        let file_reader = FileReader::new(&[elf_directory]);

        let epoch_schedule = EpochSchedule::custom(slots_per_epoch, slots_per_epoch, false);

        let deactivate_list = migration_targets
            .iter()
            .map(|mt| mt.feature_id)
            .collect::<Vec<_>>();

        let accounts = migration_targets.iter().flat_map(|mt| {
            [
                (mt.feature_id, staged_feature_account()),
                (mt.buffer_address, buffer_account(&file_reader, mt.elf_name)),
            ]
        });

        let bpf_programs = &[UpgradeableProgramInfo {
            program_id: cbm_program_activator::id(),
            loader: bpf_loader_upgradeable::id(),
            program_path: elf_path(elf_directory, "cbm_program_activator"),
            upgrade_authority: Pubkey::new_unique(),
        }];

        let (test_validator, payer) = TestValidatorGenesis::default()
            .epoch_schedule(epoch_schedule)
            .deactivate_features(&deactivate_list)
            .add_accounts(accounts)
            .add_upgradeable_programs_with_path(bpf_programs)
            .rpc_config(JsonRpcConfig {
                enable_rpc_transaction_history: true,
                ..JsonRpcConfig::default_for_test()
            })
            .start_async()
            .await;

        Self {
            test_validator,
            payer,
            slots_per_epoch,
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
fn buffer_account(file_reader: &FileReader, elf_name: &str) -> AccountSharedData {
    let elf = file_reader.load_program_elf(elf_name);

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
        .copy_from_slice(&elf);
    account
}

fn elf_path(elf_dir: &str, program_name: &str) -> PathBuf {
    PathBuf::from(elf_dir).join(format!("{}.so", program_name))
}

pub fn progress_bar() -> ProgressBar {
    let bar = ProgressBar::new(100);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.green/blue}] ({pos}%) {msg}")
            .unwrap()
            .progress_chars("=>-"),
    );
    bar
}
