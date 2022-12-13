use anyhow::Result;
use clap::Parser;
use starknet::providers::jsonrpc::{HttpTransport, JsonRpcClient};

use crate::JsonRpcArgs;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct BlockNumber {
    #[clap(flatten)]
    jsonrpc: JsonRpcArgs,
}

impl BlockNumber {
    pub async fn run(self) -> Result<()> {
        let jsonrpc_client = JsonRpcClient::new(HttpTransport::new(self.jsonrpc.rpc));

        let block_number = jsonrpc_client.block_number().await?;

        println!("{block_number}");

        Ok(())
    }
}
