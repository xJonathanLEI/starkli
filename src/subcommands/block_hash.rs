use anyhow::Result;
use clap::Parser;
use starknet::providers::Provider;

use crate::{verbosity::VerbosityArgs, ProviderArgs};

#[derive(Debug, Parser)]
pub struct BlockHash {
    #[clap(flatten)]
    provider: ProviderArgs,
    #[clap(flatten)]
    verbosity: VerbosityArgs,
}

impl BlockHash {
    pub async fn run(self) -> Result<()> {
        self.verbosity.setup_logging();

        let provider = self.provider.into_provider();

        let block = provider.block_hash_and_number().await?;

        println!("{:#064x}", block.block_hash);

        Ok(())
    }
}
