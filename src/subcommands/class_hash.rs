use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use starknet::core::types::ContractArtifact;

#[derive(Debug, Parser)]
pub struct ClassHash {
    #[clap(help = "Path to contract artifact file")]
    file: PathBuf,
}

impl ClassHash {
    pub fn run(self) -> Result<()> {
        let contract_artifact: ContractArtifact =
            serde_json::from_reader(std::fs::File::open(self.file)?)?;

        let class_hash = match contract_artifact {
            ContractArtifact::SierraClass(class) => class.class_hash()?,
            ContractArtifact::CompiledClass(class) => class.class_hash()?,
            ContractArtifact::LegacyClass(class) => class.class_hash()?,
        };
        println!("{class_hash:#064x}");

        Ok(())
    }
}
