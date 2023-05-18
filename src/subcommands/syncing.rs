use anyhow::Result;
use clap::Parser;
use colored_json::{ColorMode, Output};
use starknet::{
    core::types::SyncStatusType,
    providers::{
        jsonrpc::{HttpTransport, JsonRpcClient},
        Provider,
    },
};

use crate::JsonRpcArgs;

#[derive(Debug, Parser)]
pub struct Syncing {
    #[clap(flatten)]
    jsonrpc: JsonRpcArgs,
}

impl Syncing {
    pub async fn run(self) -> Result<()> {
        let jsonrpc_client = JsonRpcClient::new(HttpTransport::new(self.jsonrpc.rpc));

        let sync_status = jsonrpc_client.syncing().await?;

        match sync_status {
            SyncStatusType::Syncing(status) => {
                let status_json = serde_json::to_value(status)?;
                let status_json =
                    colored_json::to_colored_json(&status_json, ColorMode::Auto(Output::StdOut))?;
                println!("{status_json}");
            }
            SyncStatusType::NotSyncing => {
                println!("Not syncing");
            }
        }

        Ok(())
    }
}
