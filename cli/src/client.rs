use {
    crate::output,
    solana_client::nonblocking::rpc_client::RpcClient,
    solana_sdk::{
        commitment_config::CommitmentConfig,
        instruction::Instruction,
        pubkey::Pubkey,
        signature::{Keypair, Signature},
        signer::Signer,
        transaction::{Transaction, TransactionError},
    },
    solana_transaction_status::{option_serializer::OptionSerializer, UiTransactionEncoding},
};

const EXPECTED_RETURN_DATA: &[u8] = &[7; 32];

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

    async fn transaction(&self, program_id: &Pubkey) -> Transaction {
        let recent_blockhash = self.rpc_client.get_latest_blockhash().await.unwrap();
        Transaction::new_signed_with_payer(
            &[Instruction {
                program_id: *program_id,
                accounts: vec![],
                data: vec![],
            }],
            Some(&self.fee_payer.pubkey()),
            &[&self.fee_payer],
            recent_blockhash,
        )
    }

    async fn send_transaction(
        &self,
        program_id: &Pubkey,
    ) -> Result<Signature, Option<TransactionError>> {
        output::sending_transaction(program_id);
        let transaction = self.transaction(program_id).await;
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

    pub async fn check_return_data(&self, signature: &Signature) -> Result<(), ClientError> {
        self.confirm_transaction(signature).await?;
        let transaction = self
            .rpc_client
            .get_transaction(signature, UiTransactionEncoding::Base64)
            .await?;
        if let Some(meta) = transaction.transaction.meta {
            if let OptionSerializer::Some(ui_return_data) = meta.return_data {
                let return_data_base64 = ui_return_data.data.0;
                #[allow(deprecated)]
                let return_data_bytes = base64::decode(return_data_base64)?;
                if return_data_bytes == EXPECTED_RETURN_DATA {
                    output::expected_return_data(&return_data_bytes);
                }
            }
        }
        Ok(())
    }

    pub async fn expect_success(&self, program_id: &Pubkey) -> Result<(), ClientError> {
        match self.send_transaction(program_id).await {
            Ok(signature) => {
                output::expect_success(&signature);
                self.check_return_data(&signature).await
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

    pub async fn expect_failure_program_missing(
        &self,
        program_id: &Pubkey,
    ) -> Result<(), ClientError> {
        if let Err(err) = self.send_transaction(program_id).await {
            if let Some(transaction_err) = err {
                if let TransactionError::ProgramAccountNotFound = transaction_err {
                    output::expect_failure_program_missing(&transaction_err);
                    return Ok(());
                } else {
                    output::err_unexpected_transaction_error(&transaction_err);
                }
            } else {
                output::err_unexpected_other_error();
            }
            Err(output::get_test_terminated_err())
        } else {
            output::err_expected_failure_program_missing_but_got_success();
            Err(output::get_test_terminated_err())
        }
    }
}
