//! Tests taken directly from the Config program's test suite.

use {
    super::TestContext,
    bincode::{deserialize, serialized_size},
    serde::{Deserialize, Serialize},
    solana_config_program::{
        instruction as config_instruction,
        state::{get_config_data, ConfigKeys, ConfigState},
    },
    solana_sdk::{
        account::ReadableAccount,
        instruction::InstructionError,
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        transaction::{Transaction, TransactionError},
    },
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct MyConfig {
    pub item: u64,
}
impl Default for MyConfig {
    fn default() -> Self {
        Self { item: 123_456_789 }
    }
}
impl MyConfig {
    pub fn new(item: u64) -> Self {
        Self { item }
    }
}

impl ConfigState for MyConfig {
    fn max_space() -> u64 {
        serialized_size(&Self::default()).unwrap()
    }
}

fn get_config_space(key_len: usize) -> usize {
    let entry_size = bincode::serialized_size(&(Pubkey::default(), true)).unwrap() as usize;
    bincode::serialized_size(&(ConfigKeys::default(), MyConfig::default())).unwrap() as usize
        + key_len * entry_size
}

async fn create_config_account(
    context: &TestContext<'_>,
    config_keypair: &Keypair,
    keys: Vec<(Pubkey, bool)>,
) {
    let payer = &context.payer;
    let last_blockhash = context.rpc_client.get_latest_blockhash().await.unwrap();

    let space = get_config_space(keys.len());
    let lamports = context
        .rpc_client
        .get_minimum_balance_for_rent_exemption(space)
        .await
        .unwrap();
    let instructions = config_instruction::create_account::<MyConfig>(
        &payer.pubkey(),
        &config_keypair.pubkey(),
        lamports,
        keys,
    );

    context
        .rpc_client
        .send_and_confirm_transaction(&Transaction::new_signed_with_payer(
            &instructions,
            Some(&payer.pubkey()),
            &[payer, &config_keypair],
            last_blockhash,
        ))
        .await
        .unwrap();
}

async fn test_process_create_ok(context: &TestContext<'_>) {
    let config_keypair = Keypair::new();
    create_config_account(context, &config_keypair, vec![]).await;
    let config_account = context
        .rpc_client
        .get_account(&config_keypair.pubkey())
        .await
        .unwrap();
    assert_eq!(
        Some(MyConfig::default()),
        deserialize(get_config_data(config_account.data()).unwrap()).ok()
    );
}

async fn test_process_store_ok(context: &TestContext<'_>) {
    let last_blockhash = context.rpc_client.get_latest_blockhash().await.unwrap();

    let config_keypair = Keypair::new();
    let keys = vec![];
    let my_config = MyConfig::new(42);

    create_config_account(context, &config_keypair, keys.clone()).await;
    let instruction = config_instruction::store(&config_keypair.pubkey(), true, keys, &my_config);
    let payer = &context.payer;

    context
        .rpc_client
        .send_and_confirm_transaction(&Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[payer, &config_keypair],
            last_blockhash,
        ))
        .await
        .unwrap();

    let config_account = context
        .rpc_client
        .get_account(&config_keypair.pubkey())
        .await
        .unwrap();
    assert_eq!(
        Some(my_config),
        deserialize(get_config_data(config_account.data()).unwrap()).ok()
    );
}

async fn test_process_store_fail_instruction_data_too_large(context: &TestContext<'_>) {
    let last_blockhash = context.rpc_client.get_latest_blockhash().await.unwrap();

    let config_keypair = Keypair::new();
    let keys = vec![];
    let my_config = MyConfig::new(42);

    create_config_account(context, &config_keypair, keys.clone()).await;
    let mut instruction =
        config_instruction::store(&config_keypair.pubkey(), true, keys, &my_config);
    instruction.data = vec![0; 123]; // <-- Replace data with a vector that's too large
    let payer = &context.payer;

    let err = context
        .rpc_client
        .send_and_confirm_transaction(&Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[payer, &config_keypair],
            last_blockhash,
        ))
        .await
        .unwrap_err()
        .get_transaction_error()
        .unwrap();
    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::InvalidInstructionData)
    );
}

async fn test_process_store_fail_account0_not_signer(context: &TestContext<'_>) {
    let last_blockhash = context.rpc_client.get_latest_blockhash().await.unwrap();

    let config_keypair = Keypair::new();
    let keys = vec![];
    let my_config = MyConfig::new(42);

    create_config_account(context, &config_keypair, keys.clone()).await;
    let mut instruction =
        config_instruction::store(&config_keypair.pubkey(), true, keys, &my_config);
    let payer = &context.payer;

    instruction.accounts[0].is_signer = false; // <----- not a signer

    let err = context
        .rpc_client
        .send_and_confirm_transaction(&Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer],
            last_blockhash,
        ))
        .await
        .unwrap_err()
        .get_transaction_error()
        .unwrap();
    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::MissingRequiredSignature)
    );
}

async fn test_process_store_with_additional_signers(context: &TestContext<'_>) {
    let last_blockhash = context.rpc_client.get_latest_blockhash().await.unwrap();

    let config_keypair = Keypair::new();

    let pubkey = Pubkey::new_unique();
    let signer0 = Keypair::new();
    let signer1 = Keypair::new();
    let keys = vec![
        (pubkey, false),
        (signer0.pubkey(), true),
        (signer1.pubkey(), true),
    ];
    let my_config = MyConfig::new(42);

    create_config_account(context, &config_keypair, keys.clone()).await;
    let instruction =
        config_instruction::store(&config_keypair.pubkey(), true, keys.clone(), &my_config);
    let payer = &context.payer;

    context
        .rpc_client
        .send_and_confirm_transaction(&Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[payer, &config_keypair, &signer0, &signer1],
            last_blockhash,
        ))
        .await
        .unwrap();

    let config_account = context
        .rpc_client
        .get_account(&config_keypair.pubkey())
        .await
        .unwrap();
    let config_state: ConfigKeys = deserialize(config_account.data()).unwrap();
    assert_eq!(config_state.keys, keys);
    assert_eq!(
        Some(my_config),
        deserialize(get_config_data(config_account.data()).unwrap()).ok()
    );
}

pub async fn test_suite(context: &TestContext<'_>) {
    test_process_create_ok(context).await;
    test_process_store_ok(context).await;
    test_process_store_fail_instruction_data_too_large(context).await;
    test_process_store_fail_account0_not_signer(context).await;
    test_process_store_with_additional_signers(context).await;
}
