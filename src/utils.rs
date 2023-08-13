use std::time::Duration;

use anyhow::Result;
use bigdecimal::{BigDecimal, Zero};
use colored::Colorize;
use num_integer::Integer;
use regex::Regex;
use starknet::{
    core::types::{BlockId, BlockTag, ExecutionResult, FieldElement, StarknetError},
    providers::{MaybeUnknownErrorCode, Provider, ProviderError, StarknetErrorWithMessage},
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
            Ok(receipt) => {
                // With JSON-RPC, once we get a receipt, the transaction must have been confirmed.
                // Rejected transactions simply aren't available. This needs to be changed once we
                // implement the sequencer fallback.

                match receipt.execution_result() {
                    ExecutionResult::Succeeded => {
                        eprintln!(
                            "Transaction {} confirmed",
                            format!("{:#064x}", transaction_hash).bright_yellow()
                        );

                        return Ok(());
                    }
                    ExecutionResult::Reverted { reason } => {
                        return Err(anyhow::anyhow!("transaction reverted: {}", reason));
                    }
                }
            }
            Err(ProviderError::StarknetError(StarknetErrorWithMessage {
                code: MaybeUnknownErrorCode::Known(StarknetError::TransactionHashNotFound),
                ..
            })) => {
                eprintln!("Transaction not confirmed yet...");
            }
            Err(err) => return Err(err.into()),
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
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

#[allow(clippy::comparison_chain)]
pub fn bigdecimal_to_felt<D>(dec: &BigDecimal, decimals: D) -> Result<FieldElement>
where
    D: Into<i64>,
{
    let decimals: i64 = decimals.into();

    // Scale the bigint part up or down
    let (bigint, exponent) = dec.as_bigint_and_exponent();

    let mut biguint = match bigint.to_biguint() {
        Some(value) => value,
        None => anyhow::bail!("too many decimal places"),
    };

    if exponent < decimals {
        for _ in 0..(decimals - exponent) {
            biguint *= 10u32;
        }
    } else if exponent > decimals {
        for _ in 0..(exponent - decimals) {
            let (quotient, remainder) = biguint.div_rem(&10u32.into());
            if !remainder.is_zero() {
                anyhow::bail!("too many decimal places")
            }
            biguint = quotient;
        }
    }

    Ok(FieldElement::from_byte_slice_be(&biguint.to_bytes_be())?)
}
