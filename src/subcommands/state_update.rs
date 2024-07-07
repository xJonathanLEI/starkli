use anyhow::Result;
use clap::Parser;
use colored_json::{ColorMode, Output};
use starknet::{core::types::BlockId, providers::Provider};

use crate::{block_id::BlockIdParser, verbosity::VerbosityArgs, ProviderArgs};

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

        let update_json = serde_json::to_value(provider.get_state_update(self.block_id).await?)?;

        let update_json =
            colored_json::to_colored_json(&update_json, ColorMode::Auto(Output::StdOut))?;
        println!("{update_json}");

        Ok(())
    }
}
