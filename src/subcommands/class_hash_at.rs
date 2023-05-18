use anyhow::Result;
use clap::Parser;
use starknet::{
    core::types::{BlockId, BlockTag, FieldElement},
    providers::{
        jsonrpc::{HttpTransport, JsonRpcClient},
        Provider,
    },
};

use crate::JsonRpcArgs;

#[derive(Debug, Parser)]
pub struct ClassHashAt {
    #[clap(flatten)]
    jsonrpc: JsonRpcArgs,
    #[clap(help = "Contract address")]
    address: String,
}

impl ClassHashAt {
    pub async fn run(self) -> Result<()> {
        let jsonrpc_client = JsonRpcClient::new(HttpTransport::new(self.jsonrpc.rpc));
        let address = FieldElement::from_hex_be(&self.address)?;

        // TODO: allow custom block
        let class_hash = jsonrpc_client
            .get_class_hash_at(BlockId::Tag(BlockTag::Latest), address)
            .await?;

        println!("{:#064x}", class_hash);

        Ok(())
    }
}
