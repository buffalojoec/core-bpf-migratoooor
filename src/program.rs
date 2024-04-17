use {
    crate::harness::{
        AddressLookupTableProgramTestHarness, ConfigProgramTestHarness, FeatureGateProgramHarness,
        Harness,
    },
    clap::ValueEnum,
    solana_sdk::pubkey::Pubkey,
    std::path::PathBuf,
};

#[derive(Clone, Debug, ValueEnum)]
pub enum Program {
    AddressLookupTable,
    Config,
    FeatureGate,
}
impl Program {
    pub fn id(&self) -> Pubkey {
        match self {
            Program::AddressLookupTable => solana_sdk::address_lookup_table::program::id(),
            Program::Config => solana_sdk::config::program::id(),
            Program::FeatureGate => solana_sdk::feature::id(),
        }
    }

    pub fn feature_id(&self) -> Pubkey {
        use solana_runtime::bank::builtins::test_only;
        match self {
            Program::AddressLookupTable => test_only::address_lookup_table_program::feature::id(),
            Program::Config => test_only::config_program::feature::id(),
            Program::FeatureGate => test_only::feature_gate_program::feature::id(),
        }
    }

    pub fn source_program_id(&self) -> Pubkey {
        use solana_runtime::bank::builtins::test_only;
        match self {
            Program::AddressLookupTable => {
                test_only::address_lookup_table_program::source_program::id()
            }
            Program::Config => test_only::config_program::source_program::id(),
            Program::FeatureGate => test_only::feature_gate_program::source_program::id(),
        }
    }

    pub fn elf_path(&self) -> PathBuf {
        PathBuf::from("./elfs/test_elf.so")
    }

    pub fn harness(&self) -> Box<dyn Harness> {
        match self {
            Program::AddressLookupTable => Box::new(AddressLookupTableProgramTestHarness),
            Program::Config => Box::new(ConfigProgramTestHarness),
            Program::FeatureGate => Box::new(FeatureGateProgramHarness),
        }
    }
}
