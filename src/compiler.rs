use std::{
    fmt::Display,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Result;
use cairo_starknet_1_0_0::{
    casm_contract_class::CasmContractClass as Cairo_1_0_0_CasmClass,
    contract_class::ContractClass as Cairo_1_0_0_Class,
};
use cairo_starknet_1_1_1::{
    casm_contract_class::CasmContractClass as Cairo_1_1_1_CasmClass,
    contract_class::ContractClass as Cairo_1_1_1_Class,
};
use cairo_starknet_2_0_2::{
    casm_contract_class::CasmContractClass as Cairo_2_0_2_CasmClass,
    contract_class::ContractClass as Cairo_2_0_2_Class,
};
use cairo_starknet_2_11_4::{
    casm_contract_class::CasmContractClass as Cairo_2_11_4_CasmClass,
    contract_class::ContractClass as Cairo_2_11_4_Class,
};
use cairo_starknet_2_3_1::{
    casm_contract_class::CasmContractClass as Cairo_2_3_1_CasmClass,
    contract_class::ContractClass as Cairo_2_3_1_Class,
};
use cairo_starknet_2_5_4::{
    casm_contract_class::CasmContractClass as Cairo_2_5_4_CasmClass,
    contract_class::ContractClass as Cairo_2_5_4_Class,
};
use cairo_starknet_2_6_4::{
    casm_contract_class::CasmContractClass as Cairo_2_6_4_CasmClass,
    contract_class::ContractClass as Cairo_2_6_4_Class,
};
use cairo_starknet_2_9_4::{
    casm_contract_class::CasmContractClass as Cairo_2_9_4_CasmClass,
    contract_class::ContractClass as Cairo_2_9_4_Class,
};
use starknet::core::types::{
    contract::{CompiledClass, SierraClass},
    Felt,
};

const MAX_BYTECODE_SIZE: usize = 180000;

#[derive(Debug)]
pub struct BuiltInCompiler;

#[derive(Debug)]
pub struct CompilerBinary {
    path: PathBuf,
}

/// Statically linked Sierra compiler versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkedCompilerVersion {
    V1_0_0,
    V1_1_1,
    V2_0_2,
    V2_3_1,
    V2_5_4,
    V2_6_4,
    V2_9_4,
    V2_11_4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaybeUnknownSierraVersion {
    Known(SierraVersion),
    Unknown { major: u8, minor: u8, patch: u8 },
}

/// Sierra bytecode versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SierraVersion {
    V1_0_0,
    V1_1_0,
    V1_2_0,
    V1_3_0,
    V1_4_0,
    V1_5_0,
    V1_6_0,
    V1_7_0,
}

impl BuiltInCompiler {
    pub fn version_for_class(class: &SierraClass) -> Result<LinkedCompilerVersion> {
        if class.sierra_program.len() < 3 {
            anyhow::bail!("invalid Sierra bytecode: too few elements");
        }

        let major: u8 = class.sierra_program[0]
            .try_into()
            .map_err(|_| anyhow::anyhow!("Sierra major version out of range"))?;
        let minor: u8 = class.sierra_program[1]
            .try_into()
            .map_err(|_| anyhow::anyhow!("Sierra minor version out of range"))?;
        let patch: u8 = class.sierra_program[2]
            .try_into()
            .map_err(|_| anyhow::anyhow!("Sierra patch version out of range"))?;

        let seirra_version = MaybeUnknownSierraVersion::new(major, minor, patch);
        match seirra_version {
            MaybeUnknownSierraVersion::Known(version) => Ok(version.into()),
            MaybeUnknownSierraVersion::Unknown {
                major,
                minor,
                patch,
            } => Err(anyhow::anyhow!(
                "unsupported Sierra version: {}.{}.{}",
                major,
                minor,
                patch
            )),
        }
    }

    pub fn compile(&self, class: &SierraClass) -> Result<Felt> {
        // We do this because the Sierra doesn't need ABI anyways. Feeding it with the ABI could
        // actually cause unnecessary deserialization errors due to ABI structure changes between
        // compiler versions.
        let mut class = class.clone();
        class.abi.clear();

        let sierra_class_json = serde_json::to_string(&class)?;

        let casm_class_json = match Self::version_for_class(&class)? {
            LinkedCompilerVersion::V1_0_0 => {
                // TODO: directly convert type without going through JSON
                let contract_class: Cairo_1_0_0_Class = serde_json::from_str(&sierra_class_json)?;

                // TODO: implement the `validate_compatible_sierra_version` call

                let casm_contract =
                    Cairo_1_0_0_CasmClass::from_contract_class(contract_class, false)?;

                serde_json::to_string(&casm_contract)?
            }
            LinkedCompilerVersion::V1_1_1 => {
                // TODO: directly convert type without going through JSON
                let contract_class: Cairo_1_1_1_Class = serde_json::from_str(&sierra_class_json)?;

                // TODO: implement the `validate_compatible_sierra_version` call

                let casm_contract =
                    Cairo_1_1_1_CasmClass::from_contract_class(contract_class, false)?;

                serde_json::to_string(&casm_contract)?
            }
            LinkedCompilerVersion::V2_0_2 => {
                // TODO: directly convert type without going through JSON
                let contract_class: Cairo_2_0_2_Class = serde_json::from_str(&sierra_class_json)?;

                // TODO: implement the `validate_compatible_sierra_version` call

                let casm_contract =
                    Cairo_2_0_2_CasmClass::from_contract_class(contract_class, false)?;

                serde_json::to_string(&casm_contract)?
            }
            LinkedCompilerVersion::V2_3_1 => {
                // TODO: directly convert type without going through JSON
                let contract_class: Cairo_2_3_1_Class = serde_json::from_str(&sierra_class_json)?;

                // TODO: implement the `validate_compatible_sierra_version` call

                let casm_contract =
                    Cairo_2_3_1_CasmClass::from_contract_class(contract_class, false)?;

                serde_json::to_string(&casm_contract)?
            }
            LinkedCompilerVersion::V2_5_4 => {
                // TODO: directly convert type without going through JSON
                let contract_class: Cairo_2_5_4_Class = serde_json::from_str(&sierra_class_json)?;

                // TODO: implement the `validate_compatible_sierra_version` call

                let casm_contract =
                    Cairo_2_5_4_CasmClass::from_contract_class(contract_class, false)?;

                serde_json::to_string(&casm_contract)?
            }
            LinkedCompilerVersion::V2_6_4 => {
                // TODO: directly convert type without going through JSON
                let contract_class: Cairo_2_6_4_Class = serde_json::from_str(&sierra_class_json)?;

                // TODO: implement the `validate_compatible_sierra_version` call

                let casm_contract = Cairo_2_6_4_CasmClass::from_contract_class(
                    contract_class,
                    false,
                    MAX_BYTECODE_SIZE,
                )?;

                serde_json::to_string(&casm_contract)?
            }
            LinkedCompilerVersion::V2_9_4 => {
                // TODO: directly convert type without going through JSON
                let contract_class: Cairo_2_9_4_Class = serde_json::from_str(&sierra_class_json)?;

                // TODO: implement the `validate_compatible_sierra_version` call

                let casm_contract = Cairo_2_9_4_CasmClass::from_contract_class(
                    contract_class,
                    false,
                    MAX_BYTECODE_SIZE,
                )?;

                serde_json::to_string(&casm_contract)?
            }
            LinkedCompilerVersion::V2_11_4 => {
                // TODO: directly convert type without going through JSON
                let contract_class: Cairo_2_11_4_Class = serde_json::from_str(&sierra_class_json)?;

                // TODO: implement the `validate_compatible_sierra_version` call

                let casm_contract = Cairo_2_11_4_CasmClass::from_contract_class(
                    contract_class,
                    false,
                    MAX_BYTECODE_SIZE,
                )?;

                serde_json::to_string(&casm_contract)?
            }
        };

        // TODO: directly convert type without going through JSON
        let casm_class = serde_json::from_str::<CompiledClass>(&casm_class_json)?;

        let casm_class_hash = casm_class.class_hash()?;

        Ok(casm_class_hash)
    }
}

impl CompilerBinary {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn compile(&self, class: &SierraClass) -> Result<Felt> {
        // We do this because the Sierra doesn't need ABI anyways. Feeding it with the ABI could
        // actually cause unnecessary deserialization errors due to ABI structure changes between
        // compiler versions.
        let mut class = class.clone();
        class.abi.clear();

        let mut input_file = tempfile::NamedTempFile::new()?;
        serde_json::to_writer(&mut input_file, &class)?;

        let process_output = Command::new(&self.path)
            .arg(
                input_file
                    .path()
                    .to_str()
                    .ok_or_else(|| anyhow::anyhow!("invalid temp file path"))?,
            )
            .output()?;

        if !process_output.status.success() {
            anyhow::bail!(
                "Sierra compiler process failed with exit code: {}",
                process_output.status
            );
        }

        let casm_class_json = String::from_utf8(process_output.stdout)?;

        let casm_class = serde_json::from_str::<CompiledClass>(&casm_class_json)?;

        let casm_class_hash = casm_class.class_hash()?;

        Ok(casm_class_hash)
    }
}

impl MaybeUnknownSierraVersion {
    fn new(major: u8, minor: u8, patch: u8) -> Self {
        match (major, minor, patch) {
            (1, 0, 0) => Self::Known(SierraVersion::V1_0_0),
            (1, 1, 0) => Self::Known(SierraVersion::V1_1_0),
            (1, 2, 0) => Self::Known(SierraVersion::V1_2_0),
            (1, 3, 0) => Self::Known(SierraVersion::V1_3_0),
            (1, 4, 0) => Self::Known(SierraVersion::V1_4_0),
            (1, 5, 0) => Self::Known(SierraVersion::V1_5_0),
            (1, 6, 0) => Self::Known(SierraVersion::V1_6_0),
            (1, 7, 0) => Self::Known(SierraVersion::V1_7_0),
            _ => Self::Unknown {
                major,
                minor,
                patch,
            },
        }
    }
}

impl Default for LinkedCompilerVersion {
    fn default() -> Self {
        Self::V2_11_4
    }
}

impl Display for LinkedCompilerVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinkedCompilerVersion::V1_0_0 => write!(f, "1.0.0"),
            LinkedCompilerVersion::V1_1_1 => write!(f, "1.1.1"),
            LinkedCompilerVersion::V2_0_2 => write!(f, "2.0.2"),
            LinkedCompilerVersion::V2_3_1 => write!(f, "2.3.1"),
            LinkedCompilerVersion::V2_5_4 => write!(f, "2.5.4"),
            LinkedCompilerVersion::V2_6_4 => write!(f, "2.6.4"),
            LinkedCompilerVersion::V2_9_4 => write!(f, "2.9.4"),
            LinkedCompilerVersion::V2_11_4 => write!(f, "2.11.4"),
        }
    }
}

impl From<PathBuf> for CompilerBinary {
    fn from(value: PathBuf) -> Self {
        Self { path: value }
    }
}

impl From<SierraVersion> for LinkedCompilerVersion {
    fn from(value: SierraVersion) -> Self {
        // Full Cairo-Sierra version map for reference:
        //
        // - v1.0.0: 1.0.0
        // - v1.1.0: 1.1.0
        // - v1.1.1: 1.1.0
        // - v2.0.0: 1.2.0
        // - v2.0.1: 1.2.0
        // - v2.0.2: 1.2.0
        // - v2.1.0: 1.3.0
        // - v2.1.1: 1.3.0
        // - v2.1.2: 1.3.0
        // - v2.2.0: 1.3.0
        // - v2.3.0: 1.3.0
        // - v2.3.1: 1.3.0
        // - v2.4.0: 1.4.0
        // - v2.4.1: 1.4.0
        // - v2.4.2: 1.4.0
        // - v2.4.3: 1.4.0
        // - v2.4.4: 1.4.0
        // - v2.5.0: 1.4.0
        // - v2.5.1: 1.4.0
        // - v2.5.2: 1.4.0
        // - v2.5.3: 1.4.0
        // - v2.5.4: 1.4.0
        // - v2.6.0: 1.5.0
        // - v2.6.1: 1.5.0
        // - v2.6.2: 1.5.0
        // - v2.6.3: 1.5.0
        // - v2.6.4: 1.5.0
        // - v2.7.0: 1.6.0
        // - v2.7.1: 1.6.0
        // - v2.8.0: 1.6.0
        // - v2.8.2: 1.6.0
        // - v2.8.4: 1.6.0
        // - v2.8.5: 1.6.0
        // - v2.9.0: 1.6.0
        // - v2.9.1: 1.6.0
        // - v2.9.2: 1.6.0
        // - v2.9.3: 1.6.0
        // - v2.9.4: 1.6.0
        // - v2.10.0: 1.7.0
        // - v2.10.1: 1.7.0
        // - v2.11.0: 1.7.0
        // - v2.11.1: 1.7.0
        // - v2.11.2: 1.7.0
        // - v2.11.4: 1.7.0

        match value {
            SierraVersion::V1_0_0 => Self::V1_0_0,
            SierraVersion::V1_1_0 => Self::V1_1_1,
            SierraVersion::V1_2_0 => Self::V2_0_2,
            SierraVersion::V1_3_0 => Self::V2_3_1,
            SierraVersion::V1_4_0 => Self::V2_5_4,
            SierraVersion::V1_5_0 => Self::V2_6_4,
            SierraVersion::V1_6_0 => Self::V2_9_4,
            SierraVersion::V1_7_0 => Self::V2_11_4,
        }
    }
}
