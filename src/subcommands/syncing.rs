use anyhow::Result;
use clap::Parser;
use colored_json::{ColorMode, Output};
use starknet::{core::types::SyncStatusType, providers::Provider};

use crate::{provider::ProviderArgs, verbosity::VerbosityArgs};

#[derive(Debug, Parser)]
pub struct Syncing {
    #[clap(flatten)]
    provider: ProviderArgs,
    #[clap(flatten)]
    verbosity: VerbosityArgs,
}

impl Syncing {
    pub async fn run(self) -> Result<()> {
        self.verbosity.setup_logging();

        let provider = self.provider.into_provider()?;

        let sync_status = provider.syncing().await?;

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
