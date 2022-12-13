use anyhow::Result;
use chrono::{TimeZone, Utc};
use clap::Parser;
use starknet::providers::jsonrpc::{
    models::MaybePendingBlockWithTxHashes, HttpTransport, JsonRpcClient,
};

use crate::{utils::parse_block_id, JsonRpcArgs};

#[derive(Debug, Parser)]
pub struct BlockTime {
    #[clap(flatten)]
    jsonrpc: JsonRpcArgs,
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
        default_value = "latest",
        help = "Block number, hash, or tag (latest/pending)"
    )]
    block_id: String,
}

impl BlockTime {
    pub async fn run(self) -> Result<()> {
        let jsonrpc_client = JsonRpcClient::new(HttpTransport::new(self.jsonrpc.rpc));

        let block_id = parse_block_id(&self.block_id)?;

        let block = jsonrpc_client.get_block_with_tx_hashes(&block_id).await?;
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
