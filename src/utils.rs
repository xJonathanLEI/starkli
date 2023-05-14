use std::time::Duration;

use anyhow::Result;
use colored::Colorize;
use regex::Regex;
use starknet::{
    core::types::{FieldElement, TransactionStatus},
    providers::{
        jsonrpc::models::{BlockId, BlockTag},
        Provider,
    },
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
        let tx_status = provider.get_transaction_status(transaction_hash).await?;

        match tx_status.status {
            TransactionStatus::NotReceived | TransactionStatus::Received => {
                eprintln!("Transaction not confirmed yet...");
            }
            TransactionStatus::Pending
            | TransactionStatus::AcceptedOnL2
            | TransactionStatus::AcceptedOnL1 => {
                eprintln!(
                    "Transaction {} confirmed",
                    format!("{:#064x}", transaction_hash).bright_yellow()
                );
                return Ok(());
            }
            TransactionStatus::Rejected => {
                anyhow::bail!(
                    "transaction rejected with error: {:?}",
                    tx_status.transaction_failure_reason
                );
            }
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
