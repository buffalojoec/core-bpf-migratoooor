use {
    crate::{
        client::Client, command::Command, dirs::repository_path, output, setup::get_solana_cli_path,
    },
    dotenv::dotenv,
    std::env,
};

const FEATURE_CLUSTER: &str = "development"; // Localnet

fn get_feature_keypair_path() -> String {
    dotenv().ok();
    env::var("FEATURE_KEYPAIR_PATH").expect("FEATURE_KEYPAIR_PATH variable")
}

pub async fn activate_programify_feature_gate(
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    output::starting_feature_activation();
    let solana_cli_path = get_solana_cli_path();
    let command = solana_cli_path.to_str().unwrap();
    let args = format!(
        "feature activate {} {}",
        get_feature_keypair_path(),
        FEATURE_CLUSTER,
    );
    Command::raw_command_with_dir(&command, &args, &repository_path());

    output::waiting_for_feature_activation();
    client.poll_for_next_epoch().await?;
    client.poll_slots(5).await?; // Give it a few slots after activation

    output::feature_activated();
    Ok(())
}
