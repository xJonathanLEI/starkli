use anyhow::Result;
use clap::Parser;
use starknet::providers::{
    jsonrpc::{HttpTransport, JsonRpcClient},
    Provider,
};

use crate::JsonRpcArgs;

#[derive(Debug, Parser)]
pub struct BlockNumber {
    #[clap(flatten)]
    jsonrpc: JsonRpcArgs,
}

impl BlockNumber {
    pub async fn run(self) -> Result<()> {
        let jsonrpc_client = JsonRpcClient::new(HttpTransport::new(self.jsonrpc.rpc));

        let block = jsonrpc_client.block_hash_and_number().await?;

        println!("{}", block.block_number);

        Ok(())
    }
}
