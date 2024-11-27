use anyhow::Result;
use clap::Parser;
use colored_json::{ColorMode, Output};
use starknet::{core::types::BlockId, providers::Provider};

use crate::{block_id::BlockIdParser, provider::ProviderArgs, verbosity::VerbosityArgs};

#[derive(Debug, Parser)]
pub struct BlockTraces {
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

impl BlockTraces {
    pub async fn run(self) -> Result<()> {
        self.verbosity.setup_logging();

        let provider = self.provider.into_provider()?;

        let traces_json =
            serde_json::to_value(provider.trace_block_transactions(self.block_id).await?)?;

        let traces_json =
            colored_json::to_colored_json(&traces_json, ColorMode::Auto(Output::StdOut))?;
        println!("{traces_json}");

        Ok(())
    }
}
