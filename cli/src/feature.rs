use crate::output;

// TODO: Use the Solana CLI to activate Programify Feature Gate
pub fn activate_programify_feature_gate() {
    output::starting_feature_activation();
    output::waiting_for_feature_activation();
    output::feature_activated();
}
