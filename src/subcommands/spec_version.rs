use anyhow::Result;
use clap::Parser;
use starknet::providers::Provider;

use crate::{verbosity::VerbosityArgs, ProviderArgs};

#[derive(Debug, Parser)]
pub struct SpecVersion {
    #[clap(flatten)]
    provider: ProviderArgs,
    #[clap(flatten)]
    verbosity: VerbosityArgs,
}

impl SpecVersion {
    pub async fn run(self) -> Result<()> {
        self.verbosity.setup_logging();

        let provider = self.provider.into_provider()?;

        let spec_version = provider.spec_version().await?;

        println!("{}", spec_version);

        Ok(())
    }
}
