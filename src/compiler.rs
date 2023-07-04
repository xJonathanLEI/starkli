use std::{fmt::Display, str::FromStr};

use anyhow::Result;
use cairo_starknet_1_1_0::{
    casm_contract_class::CasmContractClass as Cairo110CasmClass,
    contract_class::ContractClass as Cairo110Class,
};
use cairo_starknet_2_0_0::{
    casm_contract_class::CasmContractClass as Cairo200CasmClass,
    contract_class::ContractClass as Cairo200Class,
};
use clap::{builder::PossibleValue, Parser, ValueEnum};
use colored::Colorize;
use starknet::core::types::{
    contract::{CompiledClass, SierraClass},
    FieldElement,
};

use crate::network::{Network, NetworkSource};

#[derive(Debug, Clone, Parser)]
pub struct CompilerArgs {
    #[clap(long, help = "Statically-linked Sierra compiler version")]
    compiler_version: Option<CompilerVersion>,
}

#[derive(Debug)]
pub enum Compiler {
    BuiltIn(CompilerVersion),
}

// TODO: separate known compiler versions with linked versions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilerVersion {
    V1_1_0,
    V2_0_0,
}

impl CompilerArgs {
    pub async fn into_compiler<N>(self, network_source: N) -> Result<Compiler>
    where
        N: NetworkSource,
    {
        match self.compiler_version {
            // Always use the version directly if manually specified
            Some(version) => Ok(Compiler::BuiltIn(version)),
            None => {
                eprintln!(
                    "Sierra compiler version not specified. \
                    Attempting to automatically decide version to use..."
                );

                let network = network_source.get_network().await?;
                match network {
                    Some(network) => {
                        let auto_version = match network {
                            Network::Mainnet => CompilerVersion::V1_1_0,
                            Network::Goerli1 | Network::Goerli2 | Network::Integration => {
                                CompilerVersion::V2_0_0
                            }
                        };

                        eprintln!(
                            "Network detected: {}. \
                            Using the default compiler version for this network: {}. \
                            Use the --compiler-version flag to choose a different version.",
                            format!("{}", network).bright_yellow(),
                            format!("{}", auto_version).bright_yellow()
                        );

                        Ok(Compiler::BuiltIn(auto_version))
                    }
                    None => {
                        let default_version: CompilerVersion = Default::default();

                        eprintln!(
                            "Unknown network. Falling back to the default compiler version {}. \
                            Use the --compiler-version flag to choose a different version.",
                            format!("{}", default_version).bright_yellow()
                        );

                        Ok(Compiler::BuiltIn(default_version))
                    }
                }
            }
        }
    }
}

impl Compiler {
    pub fn version(&self) -> CompilerVersion {
        match self {
            Compiler::BuiltIn(version) => *version,
        }
    }

    pub fn compile(&self, class: &SierraClass) -> Result<FieldElement> {
        // We do this because the Sierra doesn't need ABI anyways. Feeding it with the ABI could
        // actually cause unnecessary deserialization errors due to ABI structure changes between
        // compiler versions.
        let mut class = class.clone();
        class.abi.clear();

        let sierra_class_json = serde_json::to_string(&class)?;

        let casm_class_json = match self {
            Self::BuiltIn(version) => match version {
                CompilerVersion::V1_1_0 => {
                    // TODO: directly convert type without going through JSON
                    let contract_class: Cairo110Class = serde_json::from_str(&sierra_class_json)?;

                    // TODO: implement the `validate_compatible_sierra_version` call

                    let casm_contract =
                        Cairo110CasmClass::from_contract_class(contract_class, false)?;

                    serde_json::to_string(&casm_contract)?
                }
                CompilerVersion::V2_0_0 => {
                    // TODO: directly convert type without going through JSON
                    let contract_class: Cairo200Class = serde_json::from_str(&sierra_class_json)?;

                    // TODO: implement the `validate_compatible_sierra_version` call

                    let casm_contract =
                        Cairo200CasmClass::from_contract_class(contract_class, false)?;

                    serde_json::to_string(&casm_contract)?
                }
            },
        };

        // TODO: directly convert type without going through JSON
        let casm_class = serde_json::from_str::<CompiledClass>(&casm_class_json)?;

        let casm_class_hash = casm_class.class_hash()?;

        Ok(casm_class_hash)
    }
}

impl Default for CompilerVersion {
    fn default() -> Self {
        Self::V1_1_0
    }
}

impl ValueEnum for CompilerVersion {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::V1_1_0, Self::V2_0_0]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            Self::V1_1_0 => Some(PossibleValue::new("1.1.0").alias("v1.1.0")),
            Self::V2_0_0 => Some(PossibleValue::new("2.0.0").alias("v2.0.0")),
        }
    }
}

impl FromStr for CompilerVersion {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "1.1.0" | "v1.1.0" => Ok(Self::V1_1_0),
            "2.0.0" | "v2.0.0" => Ok(Self::V2_0_0),
            _ => Err(anyhow::anyhow!("unknown version: {}", s)),
        }
    }
}

impl Display for CompilerVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilerVersion::V1_1_0 => write!(f, "1.1.0"),
            CompilerVersion::V2_0_0 => write!(f, "2.0.0"),
        }
    }
}
