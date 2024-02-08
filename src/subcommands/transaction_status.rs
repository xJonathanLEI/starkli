use anyhow::Result;
use clap::Parser;
use colored_json::{ColorMode, Output};
use starknet::{core::types::FieldElement, providers::Provider};

use crate::{verbosity::VerbosityArgs, ProviderArgs};

#[derive(Debug, Parser)]
pub struct TransactionStatus {
    #[clap(flatten)]
    provider: ProviderArgs,
    #[clap(help = "Transaction hash")]
    hash: String,
    #[clap(flatten)]
    verbosity: VerbosityArgs,
}

impl TransactionStatus {
    pub async fn run(self) -> Result<()> {
        self.verbosity.setup_logging();

        let provider = self.provider.into_provider()?;
        let transaction_hash = FieldElement::from_hex_be(&self.hash)?;

        let status = provider.get_transaction_status(transaction_hash).await?;

        let status_json = serde_json::to_value(status)?;
        let status_json =
            colored_json::to_colored_json(&status_json, ColorMode::Auto(Output::StdOut))?;
        println!("{status_json}");

        Ok(())
    }
}
