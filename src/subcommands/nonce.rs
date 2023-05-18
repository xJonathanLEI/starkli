use anyhow::Result;
use clap::Parser;
use starknet::{
    core::types::{BlockId, BlockTag, FieldElement},
    providers::Provider,
};

use crate::ProviderArgs;

#[derive(Debug, Parser)]
pub struct Nonce {
    #[clap(flatten)]
    provider: ProviderArgs,
    #[clap(help = "Contract address")]
    address: String,
}

impl Nonce {
    pub async fn run(self) -> Result<()> {
        let provider = self.provider.into_provider();
        let address = FieldElement::from_hex_be(&self.address)?;

        // TODO: allow custom block
        let nonce = provider
            .get_nonce(BlockId::Tag(BlockTag::Latest), address)
            .await?;

        println!("{}", nonce);

        Ok(())
    }
}
