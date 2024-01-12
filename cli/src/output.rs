use solana_sdk::{pubkey::Pubkey, signature::Signature, transaction::TransactionError};

pub fn start() {
    println!();
    println!();
    println!("  🚨 🚨  Starting test for Programify Feature Gate...");
    println!();
}

pub fn starting_setup() {
    println!();
    println!("  ⛓️  ⛓️   Setting up local Solana environment...");
    println!();
}

pub fn starting_local_validator() {
    println!();
    println!("  🧭 🧭  Starting local validator...");
    println!();
}

pub fn starting_build_and_deploy() {
    println!();
    println!("  🛠️  🛠️   Building and deploying program...");
    println!();
}

pub fn starting_feature_activation() {
    println!();
    println!("  🚀 🚀  Activating Programify Feature Gate feature...");
    println!();
}

pub fn waiting_for_feature_activation() {
    println!();
    println!("  ⏳ ⏳  Awaiting feature activation...");
    println!();
}

pub fn feature_activated() {
    println!();
    println!("  🟢 🟢  Feature activated!");
    println!();
}

pub fn setting_up_client() {
    println!();
    println!("  📡 📡  Setting up client...");
    println!();
}

pub fn sending_transaction(program_id: &Pubkey) {
    println!();
    println!("      📡 📡  Sending transaction...");
    println!("      📡 📡  Program ID: {}", program_id);
    println!();
}

pub fn expect_success(signature: &Signature) {
    println!();
    println!("          ✅ Got expected success: {:?}", signature);
    println!();
}

pub fn expected_return_data(data: &[u8]) {
    println!("          ✅ Got expected return data: {:?}", data);
    println!();
}

pub fn err_expect_success_got_transaction_error(err: &TransactionError) {
    println!();
    println!(
        "          ❌ Expected success, but got transaction error: {:?}",
        err
    );
    println!();
}

pub fn expect_failure_program_missing(err: &TransactionError) {
    println!();
    println!("          ✅ Got expected transaction error: {:?}", err);
    println!();
}

pub fn err_expected_failure_program_missing_but_got_success() {
    println!();
    println!("          ❌ Expected failure on program missing, but got success");
    println!();
}

pub fn err_unexpected_transaction_error(err: &TransactionError) {
    println!();
    println!("          ❌ Got unexpected transaction error: {:?}", err);
    println!();
}

pub fn err_unexpected_other_error() {
    println!();
    println!("          ❌ Got unexpected other error");
    println!();
}

pub fn get_test_terminated_err() -> Box<dyn std::error::Error> {
    "  🛑  Test terminated.  🛑".into()
}

pub fn test_concluded() {
    println!();
    println!("  📋 📋   Test complete!");
    println!();
}
