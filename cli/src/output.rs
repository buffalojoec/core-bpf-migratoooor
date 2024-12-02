use {
    solana_sdk::{feature_set::FEATURE_NAMES, pubkey::Pubkey},
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

#[rustfmt::skip]
pub fn title_stub_test(feature_id: &Pubkey, buffer_address: &Pubkey) {
    let feature_description: String = FEATURE_NAMES.get(feature_id).expect("Feature not found").chars().take(160).collect();
    
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    writeln!(&mut stdout).unwrap();
    writeln!(&mut stdout, "    =============================================").unwrap();
    writeln!(&mut stdout, "    Core BPF Migration Test: Stub Test").unwrap();
    writeln!(&mut stdout).unwrap();
    writeln!(&mut stdout, "    Description   : {}", feature_description).unwrap();
    writeln!(&mut stdout).unwrap();
    writeln!(&mut stdout, "    Feature ID    : {}", feature_id).unwrap();
    writeln!(&mut stdout).unwrap();
    writeln!(&mut stdout, "    Buffer Address: {}", buffer_address).unwrap();
    writeln!(&mut stdout, "    =============================================").unwrap();
    writeln!(&mut stdout).unwrap();
    writeln!(&mut stdout).unwrap();
    stdout.reset().unwrap();
}

#[rustfmt::skip]
pub fn title_fixtures_test(cluster: &str, buffer_address: &Pubkey) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    writeln!(&mut stdout).unwrap();
    writeln!(&mut stdout, "    =============================================").unwrap();
    writeln!(&mut stdout, "    Core BPF Migration Test: Fixtures Test").unwrap();
    writeln!(&mut stdout).unwrap();
    writeln!(&mut stdout, "    Buffer Address: {}", buffer_address).unwrap();
    writeln!(&mut stdout).unwrap();
    writeln!(&mut stdout, "    Cloning From  : {}", cluster).unwrap();
    writeln!(&mut stdout, "    =============================================").unwrap();
    writeln!(&mut stdout).unwrap();
    writeln!(&mut stdout).unwrap();
    stdout.reset().unwrap();
}

#[rustfmt::skip]
pub fn title_conformance_test(cluster: &str, buffer_address: &Pubkey) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    writeln!(&mut stdout).unwrap();
    writeln!(&mut stdout, "    =============================================").unwrap();
    writeln!(&mut stdout, "    Core BPF Migration Test: Conformance Test").unwrap();
    writeln!(&mut stdout).unwrap();
    writeln!(&mut stdout, "    Buffer Address: {}", buffer_address).unwrap();
    writeln!(&mut stdout).unwrap();
    writeln!(&mut stdout, "    Cloning From  : {}", cluster).unwrap();
    writeln!(&mut stdout, "    =============================================").unwrap();
    writeln!(&mut stdout).unwrap();
    writeln!(&mut stdout).unwrap();
    stdout.reset().unwrap();
}

pub fn output(msg: &str) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    writeln!(&mut stdout).unwrap();
    write(&mut stdout, "  [", None);
    write(&mut stdout, "CBM TEST", Some(Color::Magenta));
    write(&mut stdout, "]: ", None);
    write(&mut stdout, msg, None);
    writeln!(&mut stdout).unwrap();
    writeln!(&mut stdout).unwrap();
    stdout.reset().unwrap();
}
