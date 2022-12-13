use anyhow::Result;
use clap::Parser;
use colored_json::{ColorMode, Output};
use regex::Regex;
use starknet::{
    core::types::FieldElement,
    providers::jsonrpc::{
        models::{BlockId, BlockTag},
        HttpTransport, JsonRpcClient,
    },
};

use crate::JsonRpcArgs;

#[derive(Debug, Parser)]
pub struct GetBlock {
    #[clap(flatten)]
    jsonrpc: JsonRpcArgs,
    #[clap(
        default_value = "latest",
        help = "Block number, hash, or tag (latest/pending)"
    )]
    block_id: String,
}

impl GetBlock {
    pub async fn run(self) -> Result<()> {
        let jsonrpc_client = JsonRpcClient::new(HttpTransport::new(self.jsonrpc.rpc));

        let regex_block_number = Regex::new("^[0-9]{1,}$").unwrap();

        let block_id = if self.block_id == "latest" {
            BlockId::Tag(BlockTag::Latest)
        } else if self.block_id == "pending" {
            BlockId::Tag(BlockTag::Pending)
        } else if regex_block_number.is_match(&self.block_id) {
            BlockId::Number(self.block_id.parse::<u64>()?)
        } else {
            BlockId::Hash(FieldElement::from_hex_be(&self.block_id)?)
        };

        let block = jsonrpc_client.get_block_with_tx_hashes(&block_id).await?;

        let block_json = serde_json::to_value(&block)?;
        let block_json =
            colored_json::to_colored_json(&block_json, ColorMode::Auto(Output::StdOut))?;
        println!("{block_json}");

        Ok(())
    }
}
