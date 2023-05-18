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
pub struct Nonce {
    #[clap(flatten)]
    jsonrpc: JsonRpcArgs,
    #[clap(help = "Contract address")]
    address: String,
}

impl Nonce {
    pub async fn run(self) -> Result<()> {
        let jsonrpc_client = JsonRpcClient::new(HttpTransport::new(self.jsonrpc.rpc));
        let address = FieldElement::from_hex_be(&self.address)?;

        // TODO: allow custom block
        let nonce = jsonrpc_client
            .get_nonce(BlockId::Tag(BlockTag::Latest), address)
            .await?;

        println!("{}", nonce);

        Ok(())
    }
}
