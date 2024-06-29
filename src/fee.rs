use std::fmt::{Display, Formatter};

use anyhow::Result;
use bigdecimal::BigDecimal;
use clap::{builder::PossibleValue, Parser, ValueEnum};
use num_traits::ToPrimitive;
use starknet::{core::types::Felt, macros::felt};

use crate::utils::bigdecimal_to_felt;

#[derive(Debug, Clone, Parser)]
pub struct FeeArgs {
    #[clap(long, help = "Token to pay transaction fees in. Defaults to ETH")]
    fee_token: Option<FeeToken>,
    #[clap(long, alias = "eth-fee", help = "Shorthand for `--fee-token ETH`")]
    eth: bool,
    #[clap(long, alias = "strk-fee", help = "Shorthand for `--fee-token STRK`")]
    strk: bool,
    #[clap(long, help = "Maximum transaction fee in Ether (18 decimals)")]
    max_fee: Option<BigDecimal>,
    #[clap(long, help = "Maximum transaction fee in Wei")]
    max_fee_raw: Option<Felt>,
    #[clap(long, help = "Maximum L1 gas amount (only for STRK fee payment)")]
    gas: Option<Felt>,
    #[clap(
        long,
        help = "Maximum L1 gas price in STRK (18 decimals) (only for STRK fee payment)"
    )]
    gas_price: Option<BigDecimal>,
    #[clap(long, help = "Maximum L1 gas price in Fri (only for STRK fee payment)")]
    gas_price_raw: Option<Felt>,
    #[clap(
        long,
        help = "Only estimate transaction fee without sending transaction"
    )]
    estimate_only: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeeToken {
    Eth,
    Strk,
}

#[derive(Debug)]
pub enum FeeSetting {
    Eth(TokenFeeSetting<EthManualFeeSetting>),
    Strk(TokenFeeSetting<StrkManualFeeSetting>),
}

#[derive(Debug)]
pub enum TokenFeeSetting<M> {
    Manual(M),
    EstimateOnly,
    None,
}

#[derive(Debug)]
pub struct EthManualFeeSetting {
    pub max_fee: Felt,
}

#[derive(Debug)]
pub struct StrkManualFeeSetting {
    pub gas: Option<u64>,
    pub gas_price: Option<u128>,
}

impl FeeArgs {
    pub fn into_setting(self) -> Result<FeeSetting> {
        let fee_token = match (self.fee_token, self.eth, self.strk) {
            (None, false, false) => FeeToken::Eth,
            (Some(fee_token), false, false) => fee_token,
            (None, true, false) => FeeToken::Eth,
            (None, false, true) => FeeToken::Strk,
            _ => anyhow::bail!("invalid fee token options"),
        };

        match fee_token {
            FeeToken::Eth => {
                if self.gas.is_some() {
                    anyhow::bail!(
                        "the --gas option is not allowed when paying fees in ETH. \
                        Use --max-fee or --max-fee-raw instead for setting fees."
                    );
                }
                if self.gas_price.is_some() {
                    anyhow::bail!(
                        "the --gas-price option is not allowed when paying fees in ETH. \
                        Use --max-fee or --max-fee-raw instead for setting fees."
                    );
                }
                if self.gas_price_raw.is_some() {
                    anyhow::bail!(
                        "the --gas-price-raw option is not allowed when paying fees in ETH. \
                        Use --max-fee or --max-fee-raw instead for setting fees."
                    );
                }

                match (self.max_fee, self.max_fee_raw, self.estimate_only) {
                    (Some(max_fee), None, false) => {
                        let max_fee_felt = bigdecimal_to_felt(&max_fee, 18)?;

                        // The user is most likely making a mistake for using a max fee higher than 1 ETH
                        // TODO: allow skipping this safety check
                        if max_fee_felt > felt!("1000000000000000000") {
                            anyhow::bail!(
                                "the --max-fee value is too large. \
                                --max-fee expects a value in Ether (18 decimals). \
                                Use --max-fee-raw instead to use a raw max_fee amount in Wei."
                            )
                        }

                        Ok(FeeSetting::Eth(TokenFeeSetting::Manual(
                            EthManualFeeSetting {
                                max_fee: max_fee_felt,
                            },
                        )))
                    }
                    (None, Some(max_fee_raw), false) => Ok(FeeSetting::Eth(
                        TokenFeeSetting::Manual(EthManualFeeSetting {
                            max_fee: max_fee_raw,
                        }),
                    )),
                    (None, None, true) => Ok(FeeSetting::Eth(TokenFeeSetting::EstimateOnly)),
                    (None, None, false) => Ok(FeeSetting::Eth(TokenFeeSetting::None)),
                    _ => Err(anyhow::anyhow!(
                        "invalid fee option. \
                        At most one of --max-fee, --max-fee-raw, and --estimate-only can be used."
                    )),
                }
            }
            FeeToken::Strk => {
                if self.max_fee.is_some() {
                    anyhow::bail!(
                        "the --max-fee option is not allowed when paying fees in STRK. \
                        Use --gas, --gas-price or --gas-price-raw instead for setting fees."
                    );
                }
                if self.max_fee_raw.is_some() {
                    anyhow::bail!(
                        "the --max-fee-raw option is not allowed when paying fees in STRK. \
                        Use --gas, --gas-price or --gas-price-raw instead for setting fees."
                    );
                }

                if self.estimate_only {
                    if self.gas.is_some()
                        || self.gas_price.is_some()
                        || self.gas_price_raw.is_some()
                    {
                        anyhow::bail!(
                            "invalid fee option. --estimate-only cannot be used with --gas, \
                            --gas-price, or --gas-price-raw."
                        )
                    }

                    Ok(FeeSetting::Strk(TokenFeeSetting::EstimateOnly))
                } else {
                    let gas_override = match self.gas {
                        Some(gas) => Some(
                            gas.to_u64()
                                .ok_or_else(|| anyhow::anyhow!("gas amount out of range"))?,
                        ),
                        None => None,
                    };
                    let gas_price_override = match (self.gas_price, self.gas_price_raw) {
                        (Some(gas_price), None) => {
                            let gas_price = bigdecimal_to_felt(&gas_price, 18)?
                                .to_u128()
                                .ok_or_else(|| anyhow::anyhow!("gas price out of range"))?;

                            // The user is most likely making a mistake for using a gas price higher
                            // than 1 STRK
                            // TODO: allow skipping this safety check
                            if gas_price > 1000000000000000000 {
                                anyhow::bail!(
                                    "the --gas-price value is too large. \
                                    --gas-price expects a value in STRK (18 decimals). \
                                    Use --gas-price instead to use a raw gas_price amount in Fri."
                                )
                            }

                            Some(gas_price)
                        }
                        (None, Some(gas_price_raw)) => {
                            let gas_price = gas_price_raw
                                .to_u128()
                                .ok_or_else(|| anyhow::anyhow!("gas price out of range"))?;

                            Some(gas_price)
                        }
                        (Some(_), Some(_)) => anyhow::bail!(
                            "conflicting fee options: --gas-price and --gas-price-raw"
                        ),
                        (None, None) => None,
                    };

                    match (gas_override, gas_price_override) {
                        (None, None) => Ok(FeeSetting::Strk(TokenFeeSetting::None)),
                        (gas_override, gas_price_override) => Ok(FeeSetting::Strk(
                            TokenFeeSetting::Manual(StrkManualFeeSetting {
                                gas: gas_override,
                                gas_price: gas_price_override,
                            }),
                        )),
                    }
                }
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
        matches!(
            self,
            Self::Eth(TokenFeeSetting::EstimateOnly) | Self::Strk(TokenFeeSetting::EstimateOnly)
        )
    }
}

impl<M> TokenFeeSetting<M> {
    pub fn is_estimate_only(&self) -> bool {
        matches!(self, Self::EstimateOnly)
    }
}
