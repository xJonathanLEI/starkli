use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use starknet::core::types::{
    contract::{CompiledClass, SierraClass},
    Felt,
};

use crate::{
    compiler::{BuiltInCompiler, CompilerBinary},
    path::ExpandedPathbufParser,
};

#[derive(Debug, Clone, Parser)]
pub struct CasmArgs {
    #[clap(long, help = "Statically-linked Sierra compiler version")]
    compiler_version: Option<String>,
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
    Hash(Felt),
}

impl CasmArgs {
    pub fn into_casm_hash_source(self) -> Result<CasmHashSource> {
        match (
            self.compiler_version,
            self.compiler_path,
            self.casm_file,
            self.casm_hash,
        ) {
            (Some(_), None, None, None) => {
                // Explicitly specifying a compiler version is now ignored, as Starkli can
                // accurately infer the right version to use based on bytecode.

                eprintln!(
                    "{}",
                    "WARNING: Starkli can now accurately infer the appropriate Sierra \
                    compiler version to use. The --compiler-version option is deprecated and \
                    ignored. It will be removed in a future version."
                        .bright_magenta()
                );

                Ok(CasmHashSource::BuiltInCompiler(BuiltInCompiler))
            }
            (None, Some(compiler_path), None, None) => {
                Ok(CasmHashSource::CompilerBinary(compiler_path.into()))
            }
            (None, None, Some(casm_file), None) => Ok(CasmHashSource::CasmFile(casm_file)),
            (None, None, None, Some(casm_hash)) => Ok(CasmHashSource::Hash(casm_hash.parse()?)),
            (None, None, None, None) => Ok(CasmHashSource::BuiltInCompiler(BuiltInCompiler)),
            _ => Err(anyhow::anyhow!(
                "invalid casm hash options. \
                Use at most one of --compiler-path, --casm-file, or --casm-hash"
            )),
        }
    }
}

impl CasmHashSource {
    pub fn get_casm_hash(&self, sierra_class: &SierraClass) -> Result<Felt> {
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
