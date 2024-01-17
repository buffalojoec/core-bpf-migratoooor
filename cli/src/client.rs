use {
    crate::{output, setup::WARP_SLOT},
    solana_client::nonblocking::rpc_client::RpcClient,
    solana_sdk::{
        address_lookup_table,
        commitment_config::CommitmentConfig,
        instruction::InstructionError,
        pubkey::Pubkey,
        signature::{Keypair, Signature},
        signer::Signer,
        transaction::{Transaction, TransactionError},
    },
};

type ClientError = Box<dyn std::error::Error>;

pub struct Client {
    fee_payer: Keypair,
    rpc_client: RpcClient,
}
impl Client {
    pub async fn new() -> Result<Self, ClientError> {
        output::setting_up_client();
        let fee_payer = Keypair::new();
        let rpc_client = RpcClient::new_with_commitment(
            "http://127.0.0.1:8899".to_owned(),
            CommitmentConfig::confirmed(),
        );
        let airdrop_signature = rpc_client
            .request_airdrop(&fee_payer.pubkey(), 1000000000)
            .await?;
        let client = Self {
            fee_payer,
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

    async fn send_transaction(
        &self,
        program_id: &Pubkey,
    ) -> Result<Signature, Option<TransactionError>> {
        output::sending_transaction(program_id);
        let recent_blockhash = self.rpc_client.get_latest_blockhash().await.unwrap();
        let (mut instruction, _) = address_lookup_table::instruction::create_lookup_table(
            Pubkey::new_unique(),
            self.fee_payer.pubkey(),
            WARP_SLOT,
        );
        instruction.program_id = *program_id;
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.fee_payer.pubkey()),
            &[&self.fee_payer],
            recent_blockhash,
        );
        self.rpc_client
            .send_and_confirm_transaction(&transaction)
            .await
            .map_err(|err| err.get_transaction_error())
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
}
