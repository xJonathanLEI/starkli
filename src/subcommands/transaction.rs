use anyhow::Result;
use clap::Parser;
use starknet::{core::types::Felt, providers::Provider};

use crate::{utils::print_colored_json, verbosity::VerbosityArgs, ProviderArgs};

#[derive(Debug, Parser)]
pub struct Transaction {
    #[clap(flatten)]
    provider: ProviderArgs,
    #[clap(help = "Transaction hash")]
    hash: String,
    #[clap(flatten)]
    verbosity: VerbosityArgs,
}

impl Transaction {
    pub async fn run(self) -> Result<()> {
        self.verbosity.setup_logging();

        let provider = self.provider.into_provider()?;
        let transaction_hash = Felt::from_hex(&self.hash)?;

        let transaction = provider.get_transaction_by_hash(transaction_hash).await?;
        print_colored_json(&transaction)?;

        Ok(())
    }
}
