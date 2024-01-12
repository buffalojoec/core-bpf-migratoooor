use {
    crate::dirs::repository_path,
    serde::{Deserialize, Serialize},
    std::path::Path,
};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Command {
    Git,
    Cargo,
    Solana,
    SolanaTestValidator,
}
impl Command {
    fn command_inner(&self, args: &str, dir: &Path) {
        let command = format!("{} {}", serde_json::to_string(&self).unwrap(), args);
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(dir)
            .status()
            .expect("failed to execute process");
        assert!(status.success());
    }

    pub fn command(&self, args: &str) {
        self.command_inner(args, &repository_path())
    }

    pub fn _command_with_dir(&self, args: &str, dir: &Path) {
        self.command_inner(args, dir)
    }
}
