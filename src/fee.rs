use anyhow::Result;
use bigdecimal::BigDecimal;
use clap::Parser;
use starknet::{core::types::FieldElement, macros::felt};

use crate::utils::bigdecimal_to_felt;

#[derive(Debug, Clone, Parser)]
pub struct FeeArgs {
    #[clap(long, help = "Maximum transaction fee in Ether (18 decimals)")]
    max_fee: Option<BigDecimal>,
    #[clap(long, help = "Maximum transaction fee in Wei")]
    max_fee_raw: Option<FieldElement>,
    #[clap(
        long,
        help = "Only estimate transaction fee without sending transaction"
    )]
    estimate_only: bool,
}

#[derive(Debug)]
pub enum FeeSetting {
    Manual(FieldElement),
    EstimateOnly,
    None,
}

impl FeeArgs {
    pub fn into_setting(self) -> Result<FeeSetting> {
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

                Ok(FeeSetting::Manual(max_fee_felt))
            }
            (None, Some(max_fee_raw), false) => Ok(FeeSetting::Manual(max_fee_raw)),
            (None, None, true) => Ok(FeeSetting::EstimateOnly),
            (None, None, false) => Ok(FeeSetting::None),
            _ => Err(anyhow::anyhow!(
                "invalid fee option. \
                At most one of --max-fee, --max-fee-raw, and --estimate-only can be used."
            )),
        }
    }
}

impl FeeSetting {
    pub fn is_estimate_only(&self) -> bool {
        matches!(self, FeeSetting::EstimateOnly)
    }
}
