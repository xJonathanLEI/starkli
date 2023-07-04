use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use starknet::core::types::{contract::SierraClass, FieldElement};

use crate::{
    compiler::{BuiltInCompiler, CompilerVersion},
    network::{Network, NetworkSource},
};

#[derive(Debug, Clone, Parser)]
pub struct CasmArgs {
    #[clap(long, help = "Statically-linked Sierra compiler version")]
    compiler_version: Option<CompilerVersion>,
    #[clap(long, help = "Override Sierra compilation and use CASM hash directly")]
    casm_hash: Option<String>,
}

#[derive(Debug)]
pub enum CasmHashSource {
    BuiltInCompiler(BuiltInCompiler),
    Hash(FieldElement),
}

impl CasmArgs {
    pub async fn into_casm_hash_source<N>(self, network_source: N) -> Result<CasmHashSource>
    where
        N: NetworkSource,
    {
        match (self.compiler_version, self.casm_hash) {
            (Some(compiler_version), None) => {
                Ok(CasmHashSource::BuiltInCompiler(compiler_version.into()))
            }
            (None, Some(casm_hash)) => Ok(CasmHashSource::Hash(casm_hash.parse()?)),
            // Tries to detect compiler version if nothing provided
            (None, None) => {
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
            _ => Err(anyhow::anyhow!("invalid casm hash options")),
        }
    }
}

impl CasmHashSource {
    pub fn get_casm_hash(&self, sierra_class: &SierraClass) -> Result<FieldElement> {
        match self {
            Self::BuiltInCompiler(compiler) => compiler.compile(sierra_class),
            Self::Hash(hash) => Ok(*hash),
        }
    }
}
