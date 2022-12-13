use anyhow::Result;
use clap::Parser;
use starknet::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use url::Url;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct BlockNumber {
    #[clap(
        long = "rpc",
        env = "STARKNET_RPC",
        help = "StarkNet JSON-RPC endpoint"
    )]
    rpc: Url,
}

impl BlockNumber {
    pub async fn run(self) -> Result<()> {
        let jsonrpc_client = JsonRpcClient::new(HttpTransport::new(self.rpc));

        let block_number = jsonrpc_client.block_number().await?;

        println!("{block_number}");

        Ok(())
    }
}
