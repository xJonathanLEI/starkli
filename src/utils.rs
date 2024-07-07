use std::{io::Read, time::Duration};

use anyhow::Result;
use bigdecimal::{BigDecimal, Zero};
use colored::Colorize;
use colored_json::{ColorMode, ColoredFormatter, Output};
use flate2::read::GzDecoder;
use num_bigint::{BigInt, Sign};
use num_integer::Integer;
use regex::Regex;
use serde::Serialize;
use serde_json::ser::PrettyFormatter;
use starknet::{
    core::types::{
        contract::{
            legacy::{
                LegacyContractClass, LegacyEntrypointOffset, LegacyProgram, RawLegacyEntryPoint,
                RawLegacyEntryPoints,
            },
            AbiEntry, SierraClass, SierraClassDebugInfo,
        },
        CompressedLegacyContractClass, ExecutionResult, Felt, FlattenedSierraClass,
        LegacyContractEntryPoint, StarknetError,
    },
    providers::{Provider, ProviderError},
};

pub async fn watch_tx<P>(provider: P, transaction_hash: Felt, poll_interval: Duration) -> Result<()>
where
    P: Provider,
{
    loop {
        match provider.get_transaction_receipt(transaction_hash).await {
            Ok(receipt) => match receipt.receipt.execution_result() {
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
            },
            Err(ProviderError::StarknetError(StarknetError::TransactionHashNotFound)) => {
                eprintln!("Transaction not confirmed yet...");
            }
            Err(err) => return Err(err.into()),
        }

        tokio::time::sleep(poll_interval).await;
    }
}

pub fn parse_felt_value(felt: &str) -> Result<Felt> {
    let regex_dec_number = Regex::new("^[0-9]{1,}$").unwrap();

    if regex_dec_number.is_match(felt) {
        Ok(Felt::from_dec_str(felt)?)
    } else {
        Ok(Felt::from_hex(felt)?)
    }
}

pub fn felt_to_bigdecimal<F, D>(felt: F, decimals: D) -> BigDecimal
where
    F: AsRef<Felt>,
    D: Into<i64>,
{
    BigDecimal::new(
        BigInt::from_bytes_be(Sign::Plus, &felt.as_ref().to_bytes_be()),
        decimals.into(),
    )
}

#[allow(clippy::comparison_chain)]
pub fn bigdecimal_to_felt<D>(dec: &BigDecimal, decimals: D) -> Result<Felt>
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

    Ok(Felt::from_bytes_be_slice(&biguint.to_bytes_be()))
}

/// Prints colored JSON for any serializable value. This is better then directly calling
/// `colored_json::to_colored_json` as that method only takes `serde_json::Value`. Unfortunately,
/// converting certain values to `serde_json::Value` would result in data loss.
pub fn print_colored_json<T>(value: &T) -> Result<()>
where
    T: Serialize,
{
    let mut writer = Vec::with_capacity(128);

    if ColorMode::Auto(Output::StdOut).use_color() {
        let formatter = ColoredFormatter::new(PrettyFormatter::new());
        let mut serializer = serde_json::Serializer::with_formatter(&mut writer, formatter);
        value.serialize(&mut serializer)?;
    } else {
        let formatter = PrettyFormatter::new();
        let mut serializer = serde_json::Serializer::with_formatter(&mut writer, formatter);
        value.serialize(&mut serializer)?;
    }

    let json = unsafe {
        // `serde_json` and `colored_json` do not emit invalid UTF-8.
        String::from_utf8_unchecked(writer)
    };

    println!("{}", json);

    Ok(())
}

/// Attempts to recover a flattened Sierra class by parsing its ABI string. This works only if the
/// declared ABI string is a valid JSON representation of Seirra ABI.
pub fn parse_flattened_sierra_class(class: FlattenedSierraClass) -> Result<SierraClass> {
    Ok(SierraClass {
        sierra_program: class.sierra_program,
        sierra_program_debug_info: SierraClassDebugInfo {
            type_names: vec![],
            libfunc_names: vec![],
            user_func_names: vec![],
        },
        contract_class_version: class.contract_class_version,
        entry_points_by_type: class.entry_points_by_type,
        abi: serde_json::from_str::<Vec<AbiEntry>>(&class.abi)?,
    })
}

/// Attempts to recover a compressed legacy program.
pub fn parse_compressed_legacy_class(
    class: CompressedLegacyContractClass,
) -> Result<LegacyContractClass> {
    let mut gzip_decoder = GzDecoder::new(class.program.as_slice());
    let mut program_json = String::new();
    gzip_decoder.read_to_string(&mut program_json)?;

    let program = serde_json::from_str::<LegacyProgram>(&program_json)?;

    let is_pre_0_11_0 = match &program.compiler_version {
        Some(compiler_version) => {
            let minor_version = compiler_version
                .split('.')
                .nth(1)
                .ok_or_else(|| anyhow::anyhow!("unexpected legacy compiler version string"))?;

            let minor_version: u8 = minor_version.parse()?;
            minor_version < 11
        }
        None => true,
    };

    let abi = match class.abi {
        Some(abi) => abi.into_iter().map(|item| item.into()).collect(),
        None => vec![],
    };

    Ok(LegacyContractClass {
        abi,
        entry_points_by_type: RawLegacyEntryPoints {
            constructor: class
                .entry_points_by_type
                .constructor
                .into_iter()
                .map(|item| parse_legacy_entrypoint(&item, is_pre_0_11_0))
                .collect(),
            external: class
                .entry_points_by_type
                .external
                .into_iter()
                .map(|item| parse_legacy_entrypoint(&item, is_pre_0_11_0))
                .collect(),
            l1_handler: class
                .entry_points_by_type
                .l1_handler
                .into_iter()
                .map(|item| parse_legacy_entrypoint(&item, is_pre_0_11_0))
                .collect(),
        },
        program,
    })
}

fn parse_legacy_entrypoint(
    entrypoint: &LegacyContractEntryPoint,
    pre_0_11_0: bool,
) -> RawLegacyEntryPoint {
    RawLegacyEntryPoint {
        // This doesn't really matter as it doesn't affect class hashes. We simply try to guess as
        // close as possible.
        offset: if pre_0_11_0 {
            LegacyEntrypointOffset::U64AsHex(entrypoint.offset)
        } else {
            LegacyEntrypointOffset::U64AsInt(entrypoint.offset)
        },
        selector: entrypoint.selector,
    }
}
