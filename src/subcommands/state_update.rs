use anyhow::Result;
use clap::Parser;
use starknet::{core::types::BlockId, providers::Provider};

use crate::{
    block_id::BlockIdParser, utils::print_colored_json, verbosity::VerbosityArgs, ProviderArgs,
};

#[derive(Debug, Parser)]
pub struct StateUpdate {
    #[clap(flatten)]
    provider: ProviderArgs,
    #[clap(
        value_parser = BlockIdParser,
        default_value = "latest",
        help = "Block number, hash, or tag (latest/pending)"
    )]
    block_id: BlockId,
    #[clap(flatten)]
    verbosity: VerbosityArgs,
}

impl StateUpdate {
    pub async fn run(self) -> Result<()> {
        self.verbosity.setup_logging();

        let provider = self.provider.into_provider()?;

        print_colored_json(&provider.get_state_update(self.block_id).await?)?;

        Ok(())
    }
}
