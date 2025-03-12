use std::fmt::{Display, Formatter};

use anyhow::Result;
use bigdecimal::BigDecimal;
use clap::{builder::PossibleValue, Parser, ValueEnum};
use num_traits::ToPrimitive;
use starknet::core::types::Felt;

use crate::utils::bigdecimal_to_felt;

// The user is most likely making a mistake for using a gas price higher than 1 STRK.
const MAX_GAS_PRICE: u128 = 1000000000000000000;

#[derive(Debug, Clone, Parser)]
pub struct FeeArgs {
    #[clap(long, hide = true)]
    fee_token: Option<FeeToken>,
    #[clap(long, alias = "eth-fee", hide = true)]
    eth: bool,
    #[clap(long, alias = "strk-fee", hide = true)]
    strk: bool,
    #[clap(long, help = "Maximum L1 gas amount")]
    l1_gas: Option<Felt>,
    #[clap(long, help = "Maximum L1 gas price in STRK (18 decimals)")]
    l1_gas_price: Option<BigDecimal>,
    #[clap(long, help = "Maximum L1 gas price in Fri")]
    l1_gas_price_raw: Option<Felt>,
    #[clap(long, help = "Maximum L2 gas amount")]
    l2_gas: Option<Felt>,
    #[clap(long, help = "Maximum L2 gas price in STRK (18 decimals)")]
    l2_gas_price: Option<BigDecimal>,
    #[clap(long, help = "Maximum L2 gas price in Fri")]
    l2_gas_price_raw: Option<Felt>,
    #[clap(long, help = "Maximum L1 data gas amount")]
    l1_data_gas: Option<Felt>,
    #[clap(long, help = "Maximum L1 data gas price in STRK (18 decimals)")]
    l1_data_gas_price: Option<BigDecimal>,
    #[clap(long, help = "Maximum L1 data gas price in Fri")]
    l1_data_gas_price_raw: Option<Felt>,
    #[clap(
        long,
        help = "Only estimate transaction fee without sending transaction"
    )]
    estimate_only: bool,
}

/// This type is no longer relevant and only kept to show error messages conditionally.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeeToken {
    Eth,
    Strk,
}

#[derive(Debug)]
pub enum FeeSetting {
    Strk(TokenFeeSetting<StrkManualFeeSetting>),
}

#[derive(Debug)]
pub enum TokenFeeSetting<M> {
    Manual(M),
    EstimateOnly,
    None,
}

#[derive(Debug)]
pub struct StrkManualFeeSetting {
    pub l1_gas: Option<u64>,
    pub l1_gas_price: Option<u128>,
    pub l2_gas: Option<u64>,
    pub l2_gas_price: Option<u128>,
    pub l1_data_gas: Option<u64>,
    pub l1_data_gas_price: Option<u128>,
}

impl FeeArgs {
    pub fn into_setting(self) -> Result<FeeSetting> {
        // These 3 flags are kept (and hidden) for now only to serve an error message. Since v0.4.0
        // it's no longer possible to pay fees with ETH.
        //
        // These should be removed in a future version when the ecosystem fully transitions into
        // STRK-only fees.
        match (self.fee_token, self.eth, self.strk) {
            (None, false, false) => {
                // Not using any of the deprecated flags
            }
            (Some(FeeToken::Eth), false, false) | (None, true, false) => {
                // Trying to pay fees with ETH
                anyhow::bail!(
                    "paying transaction fees with ETH is no longer supported. \
                    To send transactions with fees in ETH, use Starkli v0.3.x instead."
                );
            }
            (Some(FeeToken::Strk), false, false) => {
                // Trying to pay fees with STRK with --fee-token
                anyhow::bail!(
                    "Starkli now always pays transaction fees in STRK. \
                    Remove the `--fee-token` option and try again."
                );
            }
            (None, false, true) => {
                // Trying to pay fees with STRK with --strk
                anyhow::bail!(
                    "Starkli now always pays transaction fees in STRK. \
                    Remove the `--strk` or `--strk-fee` option and try again."
                );
            }
            _ => {
                // Any other invalid combinations
                anyhow::bail!(
                    "Starkli now always pays transaction fees in STRK. \
                    Fee token options are no longer supported. Remove any use of `--fee-token`, \
                    `--strk`, or `--eth` and try again."
                );
            }
        };

        if self.estimate_only {
            if self.l1_gas.is_some()
                || self.l1_gas_price.is_some()
                || self.l1_gas_price_raw.is_some()
            {
                anyhow::bail!(
                    "invalid fee option. `--estimate-only` cannot be used with any of these: \
                    --l1-gas, --l1-gas-price, --l1-gas-price-raw."
                )
            }

            Ok(FeeSetting::Strk(TokenFeeSetting::EstimateOnly))
        } else {
            let l1_gas_override = resolve_gas_override(&self.l1_gas, "L1 gas amount")?;
            let l2_gas_override = resolve_gas_override(&self.l2_gas, "L2 gas amount")?;
            let l1_data_gas_override =
                resolve_gas_override(&self.l1_data_gas, "L1 data gas amount")?;

            let l1_gas_price_override = resolve_gas_price_override(
                &self.l1_gas_price,
                &self.l1_gas_price_raw,
                "--l1-gas-price",
                "--l1-gas-price-raw",
                "L1 gas price",
            )?;
            let l2_gas_price_override = resolve_gas_price_override(
                &self.l2_gas_price,
                &self.l2_gas_price_raw,
                "--l2-gas-price",
                "--l2-gas-price-raw",
                "L2 gas price",
            )?;
            let l1_data_gas_price_override = resolve_gas_price_override(
                &self.l1_data_gas_price,
                &self.l1_data_gas_price_raw,
                "--l1-gas-price",
                "--l1-gas-price-raw",
                "L1 data gas price",
            )?;

            match (
                l1_gas_override,
                l1_gas_price_override,
                l2_gas_override,
                l2_gas_price_override,
                l1_data_gas_override,
                l1_data_gas_price_override,
            ) {
                (None, None, None, None, None, None) => Ok(FeeSetting::Strk(TokenFeeSetting::None)),
                (
                    l1_gas_override,
                    l1_gas_price_override,
                    l2_gas_override,
                    l2_gas_price_override,
                    l1_data_gas_override,
                    l1_data_gas_price_override,
                ) => Ok(FeeSetting::Strk(TokenFeeSetting::Manual(
                    StrkManualFeeSetting {
                        l1_gas: l1_gas_override,
                        l1_gas_price: l1_gas_price_override,
                        l2_gas: l2_gas_override,
                        l2_gas_price: l2_gas_price_override,
                        l1_data_gas: l1_data_gas_override,
                        l1_data_gas_price: l1_data_gas_price_override,
                    },
                ))),
            }
        }
    }
}

impl ValueEnum for FeeToken {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Eth, Self::Strk]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            Self::Eth => Some(PossibleValue::new("ETH").alias("eth")),
            Self::Strk => Some(PossibleValue::new("STRK").alias("strk")),
        }
    }
}

impl Display for FeeToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eth => write!(f, "ETH"),
            Self::Strk => write!(f, "STRK"),
        }
    }
}

impl FeeSetting {
    pub fn is_estimate_only(&self) -> bool {
        matches!(self, Self::Strk(TokenFeeSetting::EstimateOnly))
    }
}

impl<M> TokenFeeSetting<M> {
    pub fn is_estimate_only(&self) -> bool {
        matches!(self, Self::EstimateOnly)
    }
}

fn resolve_gas_override(input: &Option<Felt>, display_name: &str) -> Result<Option<u64>> {
    Ok(match input {
        Some(gas) => Some(
            gas.to_u64()
                .ok_or_else(|| anyhow::anyhow!("{} out of range", display_name))?,
        ),
        None => None,
    })
}

fn resolve_gas_price_override(
    input: &Option<BigDecimal>,
    raw_input: &Option<Felt>,
    option_name: &str,
    raw_option_name: &str,
    display_name: &str,
) -> Result<Option<u128>> {
    Ok(match (input, raw_input) {
        (Some(gas_price), None) => {
            let gas_price = bigdecimal_to_felt(gas_price, 18)?
                .to_u128()
                .ok_or_else(|| anyhow::anyhow!("{} out of range", display_name))?;

            // TODO: allow skipping this safety check
            if gas_price > MAX_GAS_PRICE {
                anyhow::bail!(
                    "the {} value is too large. \
                    {} expects a value in STRK (18 decimals). \
                    Use {} instead to use a raw gas price amount in Fri.",
                    option_name,
                    option_name,
                    raw_option_name
                )
            }

            Some(gas_price)
        }
        (None, Some(gas_price_raw)) => Some(
            gas_price_raw
                .to_u128()
                .ok_or_else(|| anyhow::anyhow!("{} out of range", option_name))?,
        ),
        (Some(_), Some(_)) => {
            anyhow::bail!(
                "conflicting fee options: {} and {}",
                option_name,
                raw_option_name
            )
        }
        (None, None) => None,
    })
}
