use {
    crate::dirs::repository_path,
    serde::{Deserialize, Serialize},
    std::{path::Path, process::Stdio},
};

fn run_command(command: &str, args: &str, dir: &Path) {
    let command = format!("{} {}", command, args);
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(dir)
        .status()
        .expect("failed to execute process");
    assert!(status.success());
}

fn run_command_detached(command: &str, args: &str, dir: &Path) {
    let command = format!("{} {}", command, args);
    std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(dir)
        .stdout(Stdio::null())
        .spawn()
        .expect("failed to execute process");
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Command {
    Alias,
    Git,
    Cargo,
    Solana,
}
impl Command {
    pub fn command(&self, args: &str) {
        run_command(
            &serde_json::to_string(&self).unwrap(),
            args,
            &repository_path(),
        )
    }

    pub fn command_with_dir(&self, args: &str, dir: &Path) {
        run_command(&serde_json::to_string(&self).unwrap(), args, dir)
    }

    pub fn raw_command_with_dir(command: &str, args: &str, dir: &Path) {
        run_command(command, args, dir)
    }

    pub fn raw_command_detached_with_dir(command: &str, args: &str, dir: &Path) {
        run_command_detached(command, args, dir)
    }
}
