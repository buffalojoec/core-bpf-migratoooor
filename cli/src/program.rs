//! Program wrapper.

use {solana_sdk::pubkey::Pubkey, std::str::FromStr};

#[derive(Clone)]
pub enum Program {
    AddressLookupTable,
    Config,
    FeatureGate,
}

impl Program {
    pub const fn buffer_address(&self) -> Pubkey {
        match self {
            Self::AddressLookupTable => {
                solana_sdk::pubkey!("AhXWrD9BBUYcKjtpA3zuiiZG4ysbo6C6wjHo1QhERk6A")
            }
            Self::Config => solana_sdk::pubkey!("BuafH9fBv62u6XjzrzS4ZjAE8963ejqF5rt1f8Uga4Q3"),
            Self::FeatureGate => {
                solana_sdk::pubkey!("3D3ydPWvmEszrSjrickCtnyRSJm1rzbbSsZog8Ub6vLh")
            }
        }
    }

    pub fn elf_name(&self) -> String {
        format!("{}.so", self.name_snake_case())
    }

    pub const fn feature_gate(&self) -> Pubkey {
        match self {
            Self::AddressLookupTable => {
                solana_sdk::feature_set::migrate_address_lookup_table_program_to_core_bpf::ID
            }
            Self::Config => solana_sdk::feature_set::migrate_config_program_to_core_bpf::ID,
            Self::FeatureGate => {
                solana_sdk::feature_set::migrate_feature_gate_program_to_core_bpf::ID
            }
        }
    }

    pub const fn fixtures_path(&self) -> &str {
        match self {
            Self::AddressLookupTable => "instr/fixtures/address-lookup-table",
            Self::Config => "instr/fixtures/config",
            Self::FeatureGate => "instr/fixtures/feature-gate",
        }
    }

    fn name_snake_case(&self) -> &str {
        match self {
            Self::AddressLookupTable => "address_lookup_table",
            Self::Config => "config",
            Self::FeatureGate => "feature_gate",
        }
    }

    pub const fn program_id(&self) -> Pubkey {
        match self {
            Self::AddressLookupTable => solana_sdk::address_lookup_table::program::ID,
            Self::Config => solana_sdk::config::program::ID,
            Self::FeatureGate => solana_sdk::feature::ID,
        }
    }

    pub fn skip_conformance_fixtures(&self) -> Vec<&str> {
        match self {
            Self::AddressLookupTable => vec![
                "6d8f5dc4bb073f6ae72a950b5108c82b41c6347a_3246919",
                "9017cf61dc0da7aa28a0b63a058f685e87df1e9a_2789718",
                "9c02f3e6bc4f519ed342f4a017a4d7050faef079_2789718",
                "9d3983516dd9cc4d515bf05f98011f22935093a4_3246919",
                "c328874b96d05db6bacb01c6534e43c1c065f3bd_3246919",
                "e5474fe3b664271f922437a3c707dc7d537d91ec_2789718",
            ],
            Self::Config => vec![
                "04a0b782cb1f4b1be044313331edda9dfb4696d6",
                "c7ec10c03d5faadcebd32dc5b9a4086abef892ca_3157979",
                "68e8dbf0f31de69a2bd1d2c0fe9af3ba676301d6_3157979",
                "f84b5ad44f7a253ebc8056d06396694370a7fa4c_3157979",
                "8bbe900444c675cfc3fbf0f80ae2eb061e536a09",
            ],
            Self::FeatureGate => vec![],
        }
    }
}

impl FromStr for Program {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "address-lookup-table" => Ok(Self::AddressLookupTable),
            "config" => Ok(Self::Config),
            "feature-gate" => Ok(Self::FeatureGate),
            _ => Err(format!("Invalid program name: {}", s)),
        }
    }
}

impl std::fmt::Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::AddressLookupTable => "address-lookup-table",
            Self::Config => "config",
            Self::FeatureGate => "feature-gate",
        };
        write!(f, "{}", s)
    }
}
