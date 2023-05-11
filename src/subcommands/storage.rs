use anyhow::Result;
use clap::Parser;
use starknet::{
    core::types::FieldElement,
    providers::jsonrpc::{
        models::{BlockId, BlockTag},
        HttpTransport, JsonRpcClient,
    },
};

use crate::JsonRpcArgs;

#[derive(Debug, Parser)]
pub struct Storage {
    #[clap(flatten)]
    jsonrpc: JsonRpcArgs,
    #[clap(help = "Contract address")]
    address: String,
    #[clap(help = "Storage key")]
    key: String,
}

impl Storage {
    pub async fn run(self) -> Result<()> {
        let jsonrpc_client = JsonRpcClient::new(HttpTransport::new(self.jsonrpc.rpc));
        let address = FieldElement::from_hex_be(&self.address)?;
        let key = FieldElement::from_hex_be(&self.key)?;

        // TODO: allow custom block
        let value = jsonrpc_client
            .get_storage_at(address, key, &BlockId::Tag(BlockTag::Latest))
            .await?;

        println!("{:#064x}", value);

        Ok(())
    }
}
