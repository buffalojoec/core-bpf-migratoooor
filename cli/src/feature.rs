use crate::{client::Client, output};

const FEATURE_ID: &str = "<TODO>";

pub async fn activate_programify_feature_gate(
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    output::starting_feature_activation();
    let _args = format!("feature activate {}", FEATURE_ID);
    // Command::solana(&args);

    output::waiting_for_feature_activation();
    client.poll_for_next_epoch().await?;
    client.poll_slots(5).await?; // Give it a few slots after activation

    output::feature_activated();
    Ok(())
}
