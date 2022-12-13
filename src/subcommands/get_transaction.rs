use anyhow::Result;
use clap::Parser;
use starknet::{
    core::types::FieldElement,
    providers::jsonrpc::{HttpTransport, JsonRpcClient},
};
use url::Url;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct GetTransaction {
    #[clap(long = "rpc", help = "StarkNet JSON-RPC endpoint")]
    rpc: Url,
    #[clap(help = "Transaction hash")]
    hash: String,
}

impl GetTransaction {
    pub async fn run(self) -> Result<()> {
        let jsonrpc_client = JsonRpcClient::new(HttpTransport::new(self.rpc));
        let transaction_hash = FieldElement::from_hex_be(&self.hash)?;

        let transaction = jsonrpc_client
            .get_transaction_by_hash(transaction_hash)
            .await?;

        let transaction_json = serde_json::to_string_pretty(&transaction)?;
        println!("{transaction_json}");

        Ok(())
    }
}
