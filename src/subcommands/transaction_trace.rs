use anyhow::Result;
use clap::Parser;
use colored_json::{ColorMode, Output};
use starknet::{core::types::Felt, providers::Provider};

use crate::{verbosity::VerbosityArgs, ProviderArgs};

#[derive(Debug, Parser)]
pub struct TransactionTrace {
    #[clap(flatten)]
    provider: ProviderArgs,
    #[clap(help = "Transaction hash")]
    hash: String,
    #[clap(flatten)]
    verbosity: VerbosityArgs,
}

impl TransactionTrace {
    pub async fn run(self) -> Result<()> {
        self.verbosity.setup_logging();

        let provider = self.provider.into_provider()?;
        let transaction_hash: Felt = self.hash.parse()?;

        let trace = provider.trace_transaction(transaction_hash).await?;

        let trace_json = serde_json::to_value(trace)?;
        let trace_json =
            colored_json::to_colored_json(&trace_json, ColorMode::Auto(Output::StdOut))?;
        println!("{trace_json}");

        Ok(())
    }
}
