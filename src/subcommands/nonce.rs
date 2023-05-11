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
            .get_nonce(&BlockId::Tag(BlockTag::Latest), address)
            .await?;

        println!("{}", nonce);

        Ok(())
    }
}
