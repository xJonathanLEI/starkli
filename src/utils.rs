use std::time::Duration;

use anyhow::Result;
use colored::Colorize;
use regex::Regex;
use starknet::{
    core::types::{BlockId, BlockTag, FieldElement, StarknetError},
    providers::{Provider, ProviderError},
};

pub async fn watch_tx<P>(provider: P, transaction_hash: FieldElement) -> Result<()>
where
    P: Provider,
    P::Error: 'static,
{
    loop {
        // TODO: check with sequencer gateway if it's not confirmed after an extended period of
        // time, as full nodes don't have access to failed transactions and would report them
        // as `NotReceived`.
        match provider.get_transaction_receipt(transaction_hash).await {
            Ok(_) => {
                // With JSON-RPC, once we get a receipt, the transaction must have been confirmed.
                // Rejected transactions simply aren't available. This needs to be changed once we
                // implement the sequencer fallback.

                eprintln!(
                    "Transaction {} confirmed",
                    format!("{:#064x}", transaction_hash).bright_yellow()
                );
                return Ok(());
            }
            Err(ProviderError::StarknetError(StarknetError::TransactionHashNotFound)) => {
                eprintln!("Transaction not confirmed yet...");
            }
            Err(err) => return Err(err.into()),
        }

        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}

pub fn parse_block_id(id: &str) -> Result<BlockId> {
    let regex_block_number = Regex::new("^[0-9]{1,}$").unwrap();

    if id == "latest" {
        Ok(BlockId::Tag(BlockTag::Latest))
    } else if id == "pending" {
        Ok(BlockId::Tag(BlockTag::Pending))
    } else if regex_block_number.is_match(id) {
        Ok(BlockId::Number(id.parse::<u64>()?))
    } else {
        Ok(BlockId::Hash(FieldElement::from_hex_be(id)?))
    }
}

pub fn parse_felt_value(felt: &str) -> Result<FieldElement> {
    let regex_dec_number = Regex::new("^[0-9]{1,}$").unwrap();

    if regex_dec_number.is_match(felt) {
        Ok(FieldElement::from_dec_str(felt)?)
    } else {
        Ok(FieldElement::from_hex_be(felt)?)
    }
}
