use anyhow::Result;
use clap::Parser;
use starknet::{
    core::utils::parse_cairo_short_string,
    providers::{
        jsonrpc::{HttpTransport, JsonRpcClient},
        Provider,
    },
};

use crate::JsonRpcArgs;

#[derive(Debug, Parser)]
pub struct ChainId {
    #[clap(flatten)]
    jsonrpc: JsonRpcArgs,
    #[clap(long, help = "Do not show the decoded text")]
    no_decode: bool,
    #[clap(
        long,
        help = "Display the decimal instead of hexadecimal representation"
    )]
    dec: bool,
    #[clap(
        default_value = "latest",
        help = "Block number, hash, or tag (latest/pending)"
    )]
    block_id: String,
}

impl ChainId {
    pub async fn run(self) -> Result<()> {
        let jsonrpc_client = JsonRpcClient::new(HttpTransport::new(self.jsonrpc.rpc));

        let raw_chain_id = jsonrpc_client.chain_id().await?;

        println!(
            "{}{}",
            if self.dec {
                format!("{raw_chain_id}")
            } else {
                format!("{raw_chain_id:#x}")
            },
            if self.no_decode {
                "".into()
            } else {
                let decoded = parse_cairo_short_string(&raw_chain_id)?;
                format!(" ({decoded})")
            }
        );

        Ok(())
    }
}
