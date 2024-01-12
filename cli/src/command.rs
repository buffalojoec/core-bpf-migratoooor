pub struct Command;
impl Command {
    const SOLANA_ALIAS: &'static str = "solana"; // TODO

    fn run_command(command: &str, args: &str) {
        let command = format!("{} {}", command, args);
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .status()
            .expect("failed to execute process");
        assert!(status.success());
    }

    pub fn cargo(args: &str) {
        Self::run_command("cargo", args);
    }

    pub fn solana(args: &str) {
        Self::run_command(Self::SOLANA_ALIAS, args);
    }
}
