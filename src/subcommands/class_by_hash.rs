use anyhow::Result;
use clap::Parser;
use colored_json::{ColorMode, Output};
use starknet::{
    core::types::{BlockId, BlockTag, FieldElement},
    providers::{
        jsonrpc::{HttpTransport, JsonRpcClient},
        Provider,
    },
};

use crate::JsonRpcArgs;

#[derive(Debug, Parser)]
pub struct ClassByHash {
    #[clap(flatten)]
    jsonrpc: JsonRpcArgs,
    #[clap(help = "Class hash")]
    hash: String,
}

impl ClassByHash {
    pub async fn run(self) -> Result<()> {
        let jsonrpc_client = JsonRpcClient::new(HttpTransport::new(self.jsonrpc.rpc));
        let class_hash = FieldElement::from_hex_be(&self.hash)?;

        // TODO: allow custom block
        let class = jsonrpc_client
            .get_class(BlockId::Tag(BlockTag::Latest), class_hash)
            .await?;

        let class_json = serde_json::to_value(class)?;
        let class_json =
            colored_json::to_colored_json(&class_json, ColorMode::Auto(Output::StdOut))?;
        println!("{class_json}");

        Ok(())
    }
}
