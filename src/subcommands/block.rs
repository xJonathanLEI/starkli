use anyhow::Result;
use clap::Parser;
use colored_json::{ColorMode, Output};
use starknet::providers::{
    jsonrpc::{HttpTransport, JsonRpcClient},
    Provider,
};

use crate::{utils::parse_block_id, JsonRpcArgs};

#[derive(Debug, Parser)]
pub struct Block {
    #[clap(flatten)]
    jsonrpc: JsonRpcArgs,
    #[clap(long, help = "Fetch full transactions instead of hashes only")]
    full: bool,
    #[clap(
        default_value = "latest",
        help = "Block number, hash, or tag (latest/pending)"
    )]
    block_id: String,
}

impl Block {
    pub async fn run(self) -> Result<()> {
        let jsonrpc_client = JsonRpcClient::new(HttpTransport::new(self.jsonrpc.rpc));

        let block_id = parse_block_id(&self.block_id)?;

        let block_json = if self.full {
            serde_json::to_value(jsonrpc_client.get_block_with_txs(block_id).await?)?
        } else {
            serde_json::to_value(jsonrpc_client.get_block_with_tx_hashes(block_id).await?)?
        };

        let block_json =
            colored_json::to_colored_json(&block_json, ColorMode::Auto(Output::StdOut))?;
        println!("{block_json}");

        Ok(())
    }
}
