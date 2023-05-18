use anyhow::Result;
use clap::Parser;
use starknet::{
    core::types::{BlockId, BlockTag, FieldElement},
    providers::Provider,
};

use crate::ProviderArgs;

#[derive(Debug, Parser)]
pub struct ClassHashAt {
    #[clap(flatten)]
    provider: ProviderArgs,
    #[clap(help = "Contract address")]
    address: String,
}

impl ClassHashAt {
    pub async fn run(self) -> Result<()> {
        let provider = self.provider.into_provider();
        let address = FieldElement::from_hex_be(&self.address)?;

        // TODO: allow custom block
        let class_hash = provider
            .get_class_hash_at(BlockId::Tag(BlockTag::Latest), address)
            .await?;

        println!("{:#064x}", class_hash);

        Ok(())
    }
}
