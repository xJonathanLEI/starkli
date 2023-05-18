use anyhow::Result;
use clap::Parser;
use starknet::providers::{
    jsonrpc::{HttpTransport, JsonRpcClient},
    Provider,
};

use crate::JsonRpcArgs;

#[derive(Debug, Parser)]
pub struct BlockHash {
    #[clap(flatten)]
    jsonrpc: JsonRpcArgs,
}

impl BlockHash {
    pub async fn run(self) -> Result<()> {
        let jsonrpc_client = JsonRpcClient::new(HttpTransport::new(self.jsonrpc.rpc));

        let block = jsonrpc_client.block_hash_and_number().await?;

        println!("{:#064x}", block.block_hash);

        Ok(())
    }
}
