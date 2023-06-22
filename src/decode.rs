use anyhow::Result;
use starknet::core::types::FieldElement;

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
    pub async fn decode_single(&self, raw: &str) -> Result<FieldElement> {
        let decoded = self.decode(raw).await?;

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
        if let Some(addr_name) = raw.strip_prefix("addr:") {
            Ok(vec![self
                .address_book_resolver
                .resolve_name(addr_name)
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!("address book entry not found for \"{}\"", addr_name)
                })?])
        } else {
            Ok(vec![raw.parse::<FieldElement>()?])
        }
    }
}
