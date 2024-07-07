use anyhow::Result;
use chrono::{TimeZone, Utc};
use clap::Parser;
use starknet::{
    core::types::{BlockId, MaybePendingBlockWithTxHashes},
    providers::Provider,
};

use crate::{block_id::BlockIdParser, verbosity::VerbosityArgs, ProviderArgs};

#[derive(Debug, Parser)]
pub struct BlockTime {
    #[clap(flatten)]
    provider: ProviderArgs,
    #[clap(
        long,
        conflicts_with = "rfc2822",
        help = "Show block time in Unix timestamp format"
    )]
    unix: bool,
    #[clap(
        long,
        conflicts_with = "unix",
        help = "Show block time in RFC 2822 format"
    )]
    rfc2822: bool,
    #[clap(
        value_parser = BlockIdParser,
        default_value = "latest",
        help = "Block number, hash, or tag (latest/pending)"
    )]
    block_id: BlockId,
    #[clap(flatten)]
    verbosity: VerbosityArgs,
}

impl BlockTime {
    pub async fn run(self) -> Result<()> {
        self.verbosity.setup_logging();

        let provider = self.provider.into_provider()?;

        let block = provider.get_block_with_tx_hashes(self.block_id).await?;
        let timestamp = match block {
            MaybePendingBlockWithTxHashes::Block(block) => block.timestamp,
            MaybePendingBlockWithTxHashes::PendingBlock(block) => block.timestamp,
        };

        if self.unix {
            println!("{timestamp}");
        } else {
            let timestamp = Utc
                .timestamp_opt(
                    timestamp
                        .try_into()
                        .map_err(|_| anyhow::anyhow!("Block timesetamp out of range"))?,
                    0,
                )
                .unwrap();
            if self.rfc2822 {
                println!("{}", timestamp.to_rfc2822())
            } else {
                println!("{}", timestamp.to_rfc3339())
            }
        }

        Ok(())
    }
}
