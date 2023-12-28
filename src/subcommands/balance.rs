use std::sync::Arc;

use anyhow::Result;
use bigdecimal::BigDecimal;
use clap::Parser;
use num_bigint::{BigUint, ToBigInt};
use starknet::{
    core::types::{BlockId, BlockTag, FieldElement, FunctionCall},
    macros::selector,
    providers::Provider,
};

use crate::{
    address_book::AddressBookResolver, decode::FeltDecoder, verbosity::VerbosityArgs, ProviderArgs,
};

/// The default ETH address: 0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7.
const DEFAULT_ETH_ADDRESS: FieldElement = FieldElement::from_mont([
    4380532846569209554,
    17839402928228694863,
    17240401758547432026,
    418961398025637529,
]);

#[derive(Debug, Parser)]
pub struct Balance {
    #[clap(flatten)]
    provider: ProviderArgs,
    #[clap(help = "Account address")]
    account_address: String,
    #[clap(
        long,
        conflicts_with = "hex",
        help = "Display raw balance amount in integer"
    )]
    raw: bool,
    #[clap(
        long,
        conflicts_with = "raw",
        help = "Display balance amount in hexadecimal representation"
    )]
    hex: bool,
    #[clap(flatten)]
    verbosity: VerbosityArgs,
}

impl Balance {
    pub async fn run(self) -> Result<()> {
        self.verbosity.setup_logging();

        let provider = Arc::new(self.provider.into_provider()?);
        let felt_decoder = FeltDecoder::new(AddressBookResolver::new(provider.clone()));

        let account_address = felt_decoder
            .decode_single_with_addr_fallback(&self.account_address)
            .await?;

        let result = provider
            .call(
                FunctionCall {
                    contract_address: DEFAULT_ETH_ADDRESS,
                    entry_point_selector: selector!("balanceOf"),
                    calldata: vec![account_address],
                },
                BlockId::Tag(BlockTag::Pending),
            )
            .await?;

        if result.len() != 2 {
            anyhow::bail!("unexpected call result size: {}", result.len());
        }

        let low = BigUint::from_bytes_be(&result[0].to_bytes_be());
        let high = BigUint::from_bytes_be(&result[1].to_bytes_be());

        let raw_balance: BigUint = (high << 128) + low;

        if self.raw {
            println!("{}", raw_balance);
        } else if self.hex {
            println!("{:#x}", raw_balance);
        } else {
            // `to_bigint()` from `BigUint` always returns `Some`.
            let balance_dec = BigDecimal::new(raw_balance.to_bigint().unwrap(), 18);
            println!("{}", balance_dec);
        }

        Ok(())
    }
}
