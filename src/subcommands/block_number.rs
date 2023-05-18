use anyhow::Result;
use clap::Parser;
use starknet::providers::Provider;

use crate::ProviderArgs;

#[derive(Debug, Parser)]
pub struct BlockNumber {
    #[clap(flatten)]
    provider: ProviderArgs,
}

impl BlockNumber {
    pub async fn run(self) -> Result<()> {
        let provider = self.provider.into_provider();

        let block = provider.block_hash_and_number().await?;

        println!("{}", block.block_number);

        Ok(())
    }
}
