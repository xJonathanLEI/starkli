use anyhow::Result;
use clap::Parser;
use colored_json::{ColorMode, Output};
use starknet::{core::types::BlockId, providers::Provider};

use crate::{block_id::BlockIdParser, provider::ProviderArgs, verbosity::VerbosityArgs};

#[derive(Debug, Parser)]
pub struct Block {
    #[clap(flatten)]
    provider: ProviderArgs,
    #[clap(long, help = "Fetch full transactions instead of hashes only")]
    full: bool,
    #[clap(long, help = "Fetch receipts alongside transactions")]
    receipts: bool,
    #[clap(
        value_parser = BlockIdParser,
        default_value = "latest",
        help = "Block number, hash, or tag (latest/pending)"
    )]
    block_id: BlockId,
    #[clap(flatten)]
    verbosity: VerbosityArgs,
}

impl Block {
    pub async fn run(self) -> Result<()> {
        self.verbosity.setup_logging();

        let provider = self.provider.into_provider()?;

        let block_json = if self.receipts {
            serde_json::to_value(provider.get_block_with_receipts(self.block_id).await?)?
        } else if self.full {
            serde_json::to_value(provider.get_block_with_txs(self.block_id).await?)?
        } else {
            serde_json::to_value(provider.get_block_with_tx_hashes(self.block_id).await?)?
        };

        let block_json =
            colored_json::to_colored_json(&block_json, ColorMode::Auto(Output::StdOut))?;
        println!("{block_json}");

        Ok(())
    }
}
