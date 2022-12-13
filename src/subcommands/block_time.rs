use anyhow::Result;
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

        println!("{timestamp}");

        Ok(())
    }
}
