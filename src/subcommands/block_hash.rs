use anyhow::Result;
use clap::Parser;
use starknet::providers::Provider;

use crate::ProviderArgs;

#[derive(Debug, Parser)]
pub struct BlockHash {
    #[clap(flatten)]
    provider: ProviderArgs,
}

impl BlockHash {
    pub async fn run(self) -> Result<()> {
        let provider = self.provider.into_provider();

        let block = provider.block_hash_and_number().await?;

        println!("{:#064x}", block.block_hash);

        Ok(())
    }
}
