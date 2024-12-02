use anyhow::Result;
use clap::Parser;
use starknet::{core::types::SyncStatusType, providers::Provider};

use crate::{utils::print_colored_json, verbosity::VerbosityArgs, ProviderArgs};

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
                print_colored_json(&status)?;
            }
            SyncStatusType::NotSyncing => {
                println!("Not syncing");
            }
        }

        Ok(())
    }
}
