use anyhow::Result;
use clap::Parser;
use starknet::{
    core::types::{BlockId, BlockTag, FieldElement},
    providers::Provider,
};

use crate::{verbosity::VerbosityArgs, ProviderArgs};

#[derive(Debug, Parser)]
pub struct Storage {
    #[clap(flatten)]
    provider: ProviderArgs,
    #[clap(help = "Contract address")]
    address: String,
    #[clap(help = "Storage key")]
    key: String,
    #[clap(flatten)]
    verbosity: VerbosityArgs,
}

impl Storage {
    pub async fn run(self) -> Result<()> {
        self.verbosity.setup_logging();

        let provider = self.provider.into_provider();
        let address = FieldElement::from_hex_be(&self.address)?;
        let key = FieldElement::from_hex_be(&self.key)?;

        // TODO: allow custom block
        let value = provider
            .get_storage_at(address, key, BlockId::Tag(BlockTag::Pending))
            .await?;

        println!("{:#064x}", value);

        Ok(())
    }
}
