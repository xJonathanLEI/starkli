use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use starknet::core::types::{
    contract::{CompiledClass, SierraClass},
    FieldElement,
};

use crate::{
    compiler::{BuiltInCompiler, CompilerBinary, CompilerVersion},
    network::{Network, NetworkSource},
    path::ExpandedPathbufParser,
};

#[derive(Debug, Clone, Parser)]
pub struct CasmArgs {
    #[clap(long, help = "Statically-linked Sierra compiler version")]
    compiler_version: Option<CompilerVersion>,
    #[clap(
        long,
        value_parser = ExpandedPathbufParser,
        help = "Path to the starknet-sierra-compile binary"
    )]
    compiler_path: Option<PathBuf>,
    #[clap(
        long,
        value_parser = ExpandedPathbufParser,
        help = "Path to already-compiled CASM file"
    )]
    casm_file: Option<PathBuf>,
    #[clap(long, help = "Override Sierra compilation and use CASM hash directly")]
    casm_hash: Option<String>,
}

#[derive(Debug)]
pub enum CasmHashSource {
    BuiltInCompiler(BuiltInCompiler),
    CompilerBinary(CompilerBinary),
    CasmFile(PathBuf),
    Hash(FieldElement),
}

impl CasmArgs {
    pub async fn into_casm_hash_source<N>(self, network_source: N) -> Result<CasmHashSource>
    where
        N: NetworkSource,
    {
        match (
            self.compiler_version,
            self.compiler_path,
            self.casm_file,
            self.casm_hash,
        ) {
            (Some(compiler_version), None, None, None) => {
                Ok(CasmHashSource::BuiltInCompiler(compiler_version.into()))
            }
            (None, Some(compiler_path), None, None) => {
                Ok(CasmHashSource::CompilerBinary(compiler_path.into()))
            }
            (None, None, Some(casm_file), None) => Ok(CasmHashSource::CasmFile(casm_file)),
            (None, None, None, Some(casm_hash)) => Ok(CasmHashSource::Hash(casm_hash.parse()?)),
            // Tries to detect compiler version if nothing provided
            (None, None, None, None) => {
                eprintln!(
                    "Sierra compiler version not specified. \
                    Attempting to automatically decide version to use..."
                );

                let network = network_source.get_network().await?;
                match network {
                    Some(network) => {
                        let auto_version = match network {
                            Network::Goerli
                            | Network::Sepolia
                            | Network::GoerliIntegration
                            | Network::SepoliaIntegration
                            | Network::Mainnet => CompilerVersion::V2_6_2,
                        };

                        eprintln!(
                            "Network detected: {}. \
                            Using the default compiler version for this network: {}. \
                            Use the --compiler-version flag to choose a different version.",
                            format!("{}", network).bright_yellow(),
                            format!("{}", auto_version).bright_yellow()
                        );

                        Ok(CasmHashSource::BuiltInCompiler(auto_version.into()))
                    }
                    None => {
                        let default_version: CompilerVersion = Default::default();

                        eprintln!(
                            "Unknown network. Falling back to the default compiler version {}. \
                            Use the --compiler-version flag to choose a different version.",
                            format!("{}", default_version).bright_yellow()
                        );

                        Ok(CasmHashSource::BuiltInCompiler(default_version.into()))
                    }
                }
            }
            _ => Err(anyhow::anyhow!(
                "invalid casm hash options. \
                Use either --compiler-version or --casm-hash but not at the same time"
            )),
        }
    }
}

impl CasmHashSource {
    pub fn get_casm_hash(&self, sierra_class: &SierraClass) -> Result<FieldElement> {
        match self {
            Self::BuiltInCompiler(compiler) => compiler.compile(sierra_class),
            Self::CompilerBinary(compiler) => compiler.compile(sierra_class),
            Self::CasmFile(path) => {
                let mut casm_file = std::fs::File::open(path)?;
                let casm_class = serde_json::from_reader::<_, CompiledClass>(&mut casm_file)?;

                Ok(casm_class.class_hash()?)
            }
            Self::Hash(hash) => Ok(*hash),
        }
    }
}
