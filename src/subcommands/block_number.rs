use anyhow::Result;
use clap::Parser;
use starknet::providers::Provider;

use crate::{provider::ProviderArgs, verbosity::VerbosityArgs};

#[derive(Debug, Parser)]
pub struct BlockNumber {
    #[clap(flatten)]
    provider: ProviderArgs,
    #[clap(flatten)]
    verbosity: VerbosityArgs,
}

impl BlockNumber {
    pub async fn run(self) -> Result<()> {
        self.verbosity.setup_logging();

        let provider = self.provider.into_provider()?;

        let block = provider.block_hash_and_number().await?;

        println!("{}", block.block_number);

        Ok(())
    }
}
