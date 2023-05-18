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
pub struct ClassAt {
    #[clap(flatten)]
    jsonrpc: JsonRpcArgs,
    #[clap(help = "Contract address")]
    address: String,
}

impl ClassAt {
    pub async fn run(self) -> Result<()> {
        let jsonrpc_client = JsonRpcClient::new(HttpTransport::new(self.jsonrpc.rpc));
        let address = FieldElement::from_hex_be(&self.address)?;

        // TODO: allow custom block
        let class = jsonrpc_client
            .get_class_at(BlockId::Tag(BlockTag::Latest), address)
            .await?;

        let class_json = serde_json::to_value(class)?;
        let class_json =
            colored_json::to_colored_json(&class_json, ColorMode::Auto(Output::StdOut))?;
        println!("{class_json}");

        Ok(())
    }
}
