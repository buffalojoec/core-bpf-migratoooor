use {
    crate::harness::{
        AddressLookupTableProgramTestHarness, ConfigProgramTestHarness, FeatureGateProgramHarness,
        Harness, StakeProgramTestHarness,
    },
    clap::ValueEnum,
    solana_sdk::pubkey::Pubkey,
};

#[derive(Clone, Debug, ValueEnum)]
pub enum Program {
    AddressLookupTable,
    Config,
    FeatureGate,
    Stake,
}
impl Program {
    pub fn id(&self) -> Pubkey {
        match self {
            Program::AddressLookupTable => solana_sdk::address_lookup_table::program::id(),
            Program::Config => solana_sdk::config::program::id(),
            Program::FeatureGate => solana_sdk::feature::id(),
            Program::Stake => solana_sdk::stake::program::id(),
        }
    }

    pub fn harness(&self) -> Box<dyn Harness> {
        match self {
            Program::AddressLookupTable => Box::new(AddressLookupTableProgramTestHarness),
            Program::Config => Box::new(ConfigProgramTestHarness),
            Program::FeatureGate => Box::new(FeatureGateProgramHarness),
            Program::Stake => Box::new(StakeProgramTestHarness),
        }
    }
}
