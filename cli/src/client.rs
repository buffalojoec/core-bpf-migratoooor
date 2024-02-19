use {
    crate::{output, setup::WARP_SLOT},
    solana_client::{
        nonblocking::rpc_client::RpcClient, rpc_client::SerializableTransaction,
        rpc_config::RpcSendTransactionConfig,
    },
    solana_sdk::{
        address_lookup_table,
        commitment_config::CommitmentConfig,
        instruction::{AccountMeta, InstructionError},
        message::{v0, VersionedMessage},
        pubkey::Pubkey,
        signature::{Keypair, Signature},
        signer::Signer,
        transaction::{Transaction, TransactionError, VersionedTransaction},
    },
};

type ClientError = Box<dyn std::error::Error>;

pub struct Client {
    fee_payer: Keypair,
    lookup_table_address: Pubkey,
    lookup_table_keys: Vec<Pubkey>,
    rpc_client: RpcClient,
}
impl Client {
    pub async fn new() -> Result<Self, ClientError> {
        output::setting_up_client();
        let fee_payer = Keypair::new();

        let (lookup_table_address, _) =
            address_lookup_table::instruction::derive_lookup_table_address(
                &fee_payer.pubkey(),
                WARP_SLOT,
            );
        let lookup_table_keys = vec![
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
        ];

        let rpc_client = RpcClient::new_with_commitment(
            "http://127.0.0.1:8899".to_owned(),
            CommitmentConfig::finalized(),
        );

        let airdrop_signature = rpc_client
            .request_airdrop(&fee_payer.pubkey(), 100_000_000_000)
            .await?;

        let client = Self {
            fee_payer,
            lookup_table_address,
            lookup_table_keys,
            rpc_client,
        };

        client.confirm_transaction(&airdrop_signature).await?;
        Ok(client)
    }

    pub async fn poll_for_next_epoch(&self) -> Result<(), ClientError> {
        let epoch_info = self.rpc_client.get_epoch_info().await?;
        let current = epoch_info.epoch;
        loop {
            let epoch_info = self.rpc_client.get_epoch_info().await?;
            if epoch_info.epoch > current {
                return Ok(());
            }
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
    }

    pub async fn poll_slots(&self, num_slots: u64) -> Result<(), ClientError> {
        let slot = self.rpc_client.get_slot().await?;
        loop {
            let current_slot = self.rpc_client.get_slot().await?;
            if current_slot > slot + num_slots {
                return Ok(());
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }

    async fn send_and_confirm_transaction(
        &self,
        transaction: &impl SerializableTransaction,
    ) -> Result<Signature, Option<TransactionError>> {
        self.rpc_client
            .send_and_confirm_transaction(transaction)
            .await
            .map_err(|err| err.get_transaction_error())
    }

    async fn send_transaction(
        &self,
        program_id: &Pubkey,
    ) -> Result<Signature, Option<TransactionError>> {
        output::sending_transaction(program_id);
        let recent_blockhash = self.rpc_client.get_latest_blockhash().await.unwrap();

        let (mut create_instruction, _) = address_lookup_table::instruction::create_lookup_table(
            self.fee_payer.pubkey(),
            self.fee_payer.pubkey(),
            WARP_SLOT,
        );
        create_instruction.program_id = *program_id;

        let mut extend_instruction = address_lookup_table::instruction::extend_lookup_table(
            self.lookup_table_address,
            self.fee_payer.pubkey(),
            Some(self.fee_payer.pubkey()),
            self.lookup_table_keys.clone(),
        );
        extend_instruction.program_id = *program_id;

        let transaction = Transaction::new_signed_with_payer(
            &[create_instruction, extend_instruction],
            Some(&self.fee_payer.pubkey()),
            &[&self.fee_payer],
            recent_blockhash,
        );
        self.send_and_confirm_transaction(&transaction).await
    }

    async fn confirm_transaction(&self, signature: &Signature) -> Result<(), ClientError> {
        loop {
            if self
                .rpc_client
                .confirm_transaction_with_commitment(signature, CommitmentConfig::finalized())
                .await?
                .value
            {
                return Ok(());
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }

    pub async fn expect_success(&self, program_id: &Pubkey) -> Result<(), ClientError> {
        match self.send_transaction(program_id).await {
            Ok(signature) => {
                output::expect_success(&signature);
                Ok(())
            }
            Err(err) => {
                if let Some(transaction_err) = err {
                    if let TransactionError::ProgramAccountNotFound = transaction_err {
                        output::err_expect_success_got_transaction_error(&transaction_err)
                    } else {
                        output::err_unexpected_transaction_error(&transaction_err);
                    }
                } else {
                    output::err_unexpected_other_error();
                }
                Err(output::get_test_terminated_err())
            }
        }
    }

    pub async fn expect_failure_invalid_instruction(
        &self,
        program_id: &Pubkey,
    ) -> Result<(), ClientError> {
        if let Err(err) = self.send_transaction(program_id).await {
            if let Some(transaction_err) = err {
                if let TransactionError::InstructionError(0, InstructionError::InvalidArgument) =
                    transaction_err
                {
                    output::expect_failure(&transaction_err);
                    return Ok(());
                } else {
                    output::err_unexpected_transaction_error(&transaction_err);
                }
            } else {
                output::err_unexpected_other_error();
            }
            Err(output::get_test_terminated_err())
        } else {
            output::err_expected_failure_but_got_success();
            Err(output::get_test_terminated_err())
        }
    }

    pub async fn expect_failure_program_missing(
        &self,
        program_id: &Pubkey,
    ) -> Result<(), ClientError> {
        if let Err(err) = self.send_transaction(program_id).await {
            if let Some(transaction_err) = err {
                if let TransactionError::ProgramAccountNotFound = transaction_err {
                    output::expect_failure(&transaction_err);
                    return Ok(());
                } else {
                    output::err_unexpected_transaction_error(&transaction_err);
                }
            } else {
                output::err_unexpected_other_error();
            }
            Err(output::get_test_terminated_err())
        } else {
            output::err_expected_failure_but_got_success();
            Err(output::get_test_terminated_err())
        }
    }

    async fn get_address_lookup_table_account(
        &self,
    ) -> Result<address_lookup_table::AddressLookupTableAccount, ClientError> {
        let raw_account = &self
            .rpc_client
            .get_account(&self.lookup_table_address)
            .await?;
        let address_lookup_table =
            address_lookup_table::state::AddressLookupTable::deserialize(&raw_account.data)?;
        Ok(address_lookup_table::AddressLookupTableAccount {
            key: self.lookup_table_address,
            addresses: address_lookup_table.addresses.to_vec(),
        })
    }

    pub async fn expect_success_versioned_transaction(&self) -> Result<(), ClientError> {
        output::sending_versioned_transaction();
        let mut instruction = solana_sdk::system_instruction::transfer(
            &self.fee_payer.pubkey(),
            &Pubkey::new_unique(),
            10_00_000,
        );
        let extra_metas = self
            .lookup_table_keys
            .iter()
            .map(|key| AccountMeta {
                pubkey: *key,
                is_signer: false,
                is_writable: false,
            })
            .collect::<Vec<_>>();
        instruction.accounts.extend_from_slice(&extra_metas);

        let address_lookup_table_account = self.get_address_lookup_table_account().await?;
        let recent_blockhash = self.rpc_client.get_latest_blockhash().await.unwrap();
        let transaction = VersionedTransaction::try_new(
            VersionedMessage::V0(v0::Message::try_compile(
                &self.fee_payer.pubkey(),
                &[instruction],
                &[address_lookup_table_account],
                recent_blockhash,
            )?),
            &[&self.fee_payer],
        )?;

        match self.send_and_confirm_transaction(&transaction).await {
            Ok(signature) => {
                output::expect_success(&signature);
                Ok(())
            }
            Err(_) => {
                output::err_unexpected_other_error();
                Err(output::get_test_terminated_err())
            }
        }
    }
}
