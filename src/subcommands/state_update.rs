use anyhow::Result;
use clap::Parser;
use colored_json::{ColorMode, Output};
use starknet::providers::{
    jsonrpc::{HttpTransport, JsonRpcClient},
    Provider,
};

use crate::{utils::parse_block_id, JsonRpcArgs};

#[derive(Debug, Parser)]
pub struct StateUpdate {
    #[clap(flatten)]
    jsonrpc: JsonRpcArgs,
    #[clap(
        default_value = "latest",
        help = "Block number, hash, or tag (latest/pending)"
    )]
    block_id: String,
}

impl StateUpdate {
    pub async fn run(self) -> Result<()> {
        let jsonrpc_client = JsonRpcClient::new(HttpTransport::new(self.jsonrpc.rpc));

        let block_id = parse_block_id(&self.block_id)?;

        let update_json = serde_json::to_value(jsonrpc_client.get_state_update(block_id).await?)?;

        let update_json =
            colored_json::to_colored_json(&update_json, ColorMode::Auto(Output::StdOut))?;
        println!("{update_json}");

        Ok(())
    }
}
