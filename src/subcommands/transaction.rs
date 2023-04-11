use anyhow::Result;
use clap::Parser;
use colored_json::{ColorMode, Output};
use starknet::{
    core::types::FieldElement,
    providers::jsonrpc::{HttpTransport, JsonRpcClient},
};

use crate::JsonRpcArgs;

#[derive(Debug, Parser)]
pub struct Transaction {
    #[clap(flatten)]
    jsonrpc: JsonRpcArgs,
    #[clap(help = "Transaction hash")]
    hash: String,
}

impl Transaction {
    pub async fn run(self) -> Result<()> {
        let jsonrpc_client = JsonRpcClient::new(HttpTransport::new(self.jsonrpc.rpc));
        let transaction_hash = FieldElement::from_hex_be(&self.hash)?;

        let transaction = jsonrpc_client
            .get_transaction_by_hash(transaction_hash)
            .await?;

        let transaction_json = serde_json::to_value(transaction)?;
        let transaction_json =
            colored_json::to_colored_json(&transaction_json, ColorMode::Auto(Output::StdOut))?;
        println!("{transaction_json}");

        Ok(())
    }
}
