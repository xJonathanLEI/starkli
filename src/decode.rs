use anyhow::Result;
use num_bigint::BigUint;
use starknet::core::{types::FieldElement, utils::cairo_short_string_to_felt};

use crate::{address_book::AddressBookResolver, chain_id::ChainIdSource};

pub struct FeltDecoder<S> {
    address_book_resolver: AddressBookResolver<S>,
}

impl<S> FeltDecoder<S> {
    pub fn new(address_book_resolver: AddressBookResolver<S>) -> Self {
        Self {
            address_book_resolver,
        }
    }
}

impl<S> FeltDecoder<S>
where
    S: ChainIdSource,
{
    pub async fn decode_single_with_addr_fallback(&self, raw: &str) -> Result<FieldElement> {
        let decoded = self.decode_inner(raw, true).await?;

        if decoded.len() == 1 {
            Ok(decoded[0])
        } else {
            Err(anyhow::anyhow!(
                "expected 1 element but found {}",
                decoded.len()
            ))
        }
    }

    pub async fn decode(&self, raw: &str) -> Result<Vec<FieldElement>> {
        self.decode_inner(raw, false).await
    }

    async fn decode_inner(&self, raw: &str, addr_fallback: bool) -> Result<Vec<FieldElement>> {
        if let Some(addr_name) = raw.strip_prefix("addr:") {
            Ok(vec![self.resolve_addr(addr_name).await?])
        } else if let Some(u256_str) = raw.strip_prefix("u256:") {
            let bigint = if let Some(hex_str) = u256_str.strip_prefix("0x") {
                let unsigned_bytes = if hex_str.len() % 2 == 0 {
                    hex::decode(hex_str)?
                } else {
                    let mut padded = String::from("0");
                    padded.push_str(hex_str);
                    hex::decode(&padded)?
                };

                BigUint::from_bytes_be(&unsigned_bytes)
            } else {
                // If it's not prefixed with "0x" we assume decimal repr

                let digits = u256_str
                    .chars()
                    .map(|c| c.to_string().parse::<u8>())
                    .collect::<std::result::Result<Vec<_>, _>>()?;

                // All elements in `digits` must be less than 10 so this is safe
                BigUint::from_radix_be(&digits, 10).unwrap()
            };

            let u128_max_plus_1 =
                BigUint::from_bytes_be(&hex_literal::hex!("0100000000000000000000000000000000"));

            let high = &bigint / &u128_max_plus_1;
            if high >= u128_max_plus_1 {
                anyhow::bail!("u256 value out of range");
            }

            let low = &bigint % &u128_max_plus_1;

            // Unwrapping is safe as these are never out of range
            let high = FieldElement::from_byte_slice_be(&high.to_bytes_be()).unwrap();
            let low = FieldElement::from_byte_slice_be(&low.to_bytes_be()).unwrap();

            Ok(vec![low, high])
        } else if let Some(const_name) = raw.strip_prefix("const:") {
            match const_name.to_lowercase().as_str() {
                // `u256_max` is canonical and all others should be considered aliases.
                "u256_max" | "u256-max" | "u256max" | "u256::max" | "uint256_max"
                | "uint256-max" | "uint256max" | "uint256::max" => Ok(vec![
                    FieldElement::from_byte_slice_be(&hex_literal::hex!(
                        "ffffffffffffffffffffffffffffffff"
                    ))
                    .unwrap(),
                    FieldElement::from_byte_slice_be(&hex_literal::hex!(
                        "ffffffffffffffffffffffffffffffff"
                    ))
                    .unwrap(),
                ]),
                // `felt_max` is canonical and all others should be considered aliases.
                "felt_max" | "felt-max" | "felt::max" | "felt252_max" | "felt252-max"
                | "felt252::max" => Ok(vec![FieldElement::MAX]),
                _ => Err(anyhow::anyhow!("unknown constant: {}", const_name)),
            }
        } else if let Some(short_string) = raw.strip_prefix("str:") {
            Ok(vec![cairo_short_string_to_felt(short_string)?])
        } else {
            match raw.parse::<FieldElement>() {
                Ok(value) => Ok(vec![value]),
                Err(err) => {
                    if addr_fallback {
                        match self.resolve_addr(raw).await {
                            Ok(value) => Ok(vec![value]),
                            Err(_) => Err(err.into()),
                        }
                    } else {
                        Err(err.into())
                    }
                }
            }
        }
    }

    async fn resolve_addr(&self, name: &str) -> Result<FieldElement> {
        self.address_book_resolver
            .resolve_name(name)
            .await?
            .ok_or_else(|| anyhow::anyhow!("address book entry not found for \"{}\"", name))
    }
}
