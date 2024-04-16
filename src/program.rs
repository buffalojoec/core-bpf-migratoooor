use {
    crate::harness::{
        AddressLookupTableProgramTestHarness, ConfigProgramTestHarness, FeatureGateProgramHarness,
        Harness,
    },
    clap::ValueEnum,
    solana_sdk::pubkey::Pubkey,
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

    pub fn harness(&self) -> Box<dyn Harness> {
        match self {
            Program::AddressLookupTable => Box::new(AddressLookupTableProgramTestHarness),
            Program::Config => Box::new(ConfigProgramTestHarness),
            Program::FeatureGate => Box::new(FeatureGateProgramHarness),
        }
    }
}
