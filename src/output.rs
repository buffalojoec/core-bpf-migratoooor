use {
    indicatif::{ProgressBar, ProgressStyle},
    solana_sdk::pubkey::Pubkey,
    std::io::Write,
    termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor},
};

fn write(stdout: &mut StandardStream, msg: &str, color: Option<Color>) {
    if let Some(color) = color {
        stdout
            .set_color(ColorSpec::new().set_fg(Some(color)))
            .unwrap();
    } else {
        stdout.reset().unwrap();
    }
    write!(stdout, "{}", msg).unwrap();
}

pub fn title(program_id: &Pubkey) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    writeln!(&mut stdout).unwrap();
    writeln!(&mut stdout, "=============================================").unwrap();
    writeln!(&mut stdout, "    Core BPF Migration Test").unwrap();
    writeln!(&mut stdout, "    Program: {}", program_id).unwrap();
    writeln!(&mut stdout, "=============================================").unwrap();
    writeln!(&mut stdout).unwrap();
    writeln!(&mut stdout).unwrap();
    stdout.reset().unwrap();
}

pub fn output(msg: &str) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    writeln!(&mut stdout).unwrap();
    write(&mut stdout, "  [", None);
    write(&mut stdout, "CBPFM TEST", Some(Color::Magenta));
    write(&mut stdout, "]: ", None);
    write(&mut stdout, msg, None);
    writeln!(&mut stdout).unwrap();
    writeln!(&mut stdout).unwrap();
    stdout.reset().unwrap();
}

pub fn progress_bar() -> ProgressBar {
    let bar = ProgressBar::new(100);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.green/blue}] ({pos}%) {msg}")
            .unwrap()
            .progress_chars("=>-"),
    );
    bar
}
