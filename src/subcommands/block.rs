use anyhow::Result;
use clap::Parser;
use starknet::{core::types::BlockId, providers::Provider};

use crate::{
    block_id::BlockIdParser, utils::print_colored_json, verbosity::VerbosityArgs, ProviderArgs,
};

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

        if self.receipts {
            print_colored_json(&provider.get_block_with_receipts(self.block_id).await?)?;
        } else if self.full {
            print_colored_json(&provider.get_block_with_txs(self.block_id).await?)?;
        } else {
            print_colored_json(&provider.get_block_with_tx_hashes(self.block_id).await?)?;
        };

        Ok(())
    }
}
