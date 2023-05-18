use anyhow::Result;
use clap::Parser;
use colored_json::{ColorMode, Output};
use starknet::{core::types::FieldElement, providers::Provider};

use crate::ProviderArgs;

#[derive(Debug, Parser)]
pub struct TransactionReceipt {
    #[clap(flatten)]
    provider: ProviderArgs,
    #[clap(help = "Transaction hash")]
    hash: String,
}

impl TransactionReceipt {
    pub async fn run(self) -> Result<()> {
        let provider = self.provider.into_provider();
        let transaction_hash = FieldElement::from_hex_be(&self.hash)?;

        let receipt = provider.get_transaction_receipt(transaction_hash).await?;

        let receipt_json = serde_json::to_value(receipt)?;
        let receipt_json =
            colored_json::to_colored_json(&receipt_json, ColorMode::Auto(Output::StdOut))?;
        println!("{receipt_json}");

        Ok(())
    }
}
