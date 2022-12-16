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

        let class_hash = contract_artifact.class_hash()?;
        println!("{:#064x}", class_hash);

        Ok(())
    }
}
