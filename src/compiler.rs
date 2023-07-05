use std::{fmt::Display, str::FromStr};

use anyhow::Result;
use cairo_starknet_1_1_0::{
    casm_contract_class::CasmContractClass as Cairo110CasmClass,
    contract_class::ContractClass as Cairo110Class,
};
use cairo_starknet_2_0_1::{
    casm_contract_class::CasmContractClass as Cairo201CasmClass,
    contract_class::ContractClass as Cairo201Class,
};
use clap::{builder::PossibleValue, ValueEnum};
use starknet::core::types::{
    contract::{CompiledClass, SierraClass},
    FieldElement,
};

#[derive(Debug)]
pub struct BuiltInCompiler {
    version: CompilerVersion,
}

// TODO: separate known compiler versions with linked versions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilerVersion {
    V1_1_0,
    V2_0_1,
}

impl BuiltInCompiler {
    pub fn version(&self) -> CompilerVersion {
        self.version
    }

    pub fn compile(&self, class: &SierraClass) -> Result<FieldElement> {
        // We do this because the Sierra doesn't need ABI anyways. Feeding it with the ABI could
        // actually cause unnecessary deserialization errors due to ABI structure changes between
        // compiler versions.
        let mut class = class.clone();
        class.abi.clear();

        let sierra_class_json = serde_json::to_string(&class)?;

        let casm_class_json = match self.version {
            CompilerVersion::V1_1_0 => {
                // TODO: directly convert type without going through JSON
                let contract_class: Cairo110Class = serde_json::from_str(&sierra_class_json)?;

                // TODO: implement the `validate_compatible_sierra_version` call

                let casm_contract = Cairo110CasmClass::from_contract_class(contract_class, false)?;

                serde_json::to_string(&casm_contract)?
            }
            CompilerVersion::V2_0_1 => {
                // TODO: directly convert type without going through JSON
                let contract_class: Cairo201Class = serde_json::from_str(&sierra_class_json)?;

                // TODO: implement the `validate_compatible_sierra_version` call

                let casm_contract = Cairo201CasmClass::from_contract_class(contract_class, false)?;

                serde_json::to_string(&casm_contract)?
            }
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
        &[Self::V1_1_0, Self::V2_0_1]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            Self::V1_1_0 => Some(PossibleValue::new("1.1.0").alias("v1.1.0")),
            Self::V2_0_1 => Some(PossibleValue::new("2.0.1").alias("v2.0.1")),
        }
    }
}

impl FromStr for CompilerVersion {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "1.1.0" | "v1.1.0" => Ok(Self::V1_1_0),
            "2.0.1" | "v2.0.1" => Ok(Self::V2_0_1),
            _ => Err(anyhow::anyhow!("unknown version: {}", s)),
        }
    }
}

impl Display for CompilerVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilerVersion::V1_1_0 => write!(f, "1.1.0"),
            CompilerVersion::V2_0_1 => write!(f, "2.0.1"),
        }
    }
}

impl From<CompilerVersion> for BuiltInCompiler {
    fn from(value: CompilerVersion) -> Self {
        Self { version: value }
    }
}
