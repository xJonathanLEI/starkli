// Very temporary implementation of a hard-coded address book

use std::cell::OnceCell;

use anyhow::Result;
use starknet::{
    core::{chain_id, types::FieldElement},
    macros::{felt, short_string},
};

use crate::chain_id::ChainIdSource;

const CHAIN_ID_KATANA: FieldElement = felt!("0x4b4154414e41");

pub const HARDCODED_ADDRESS_BOOK: [AddressBookEntry; 6] = [
    AddressBookEntry {
        chain_id: chain_id::MAINNET,
        name: "eth",
        address: felt!("0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7"),
    },
    AddressBookEntry {
        chain_id: chain_id::TESTNET,
        name: "eth",
        address: felt!("0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7"),
    },
    AddressBookEntry {
        chain_id: short_string!("SN_SEPOLIA"),
        name: "eth",
        address: felt!("0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7"),
    },
    AddressBookEntry {
        chain_id: short_string!("SN_INTEGRATION_SEPOLIA"),
        name: "eth",
        address: felt!("0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7"),
    },
    AddressBookEntry {
        chain_id: CHAIN_ID_KATANA,
        name: "eth",
        address: felt!("0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7"),
    },
    AddressBookEntry {
        chain_id: chain_id::MAINNET,
        name: "zklend",
        address: felt!("0x04c0a5193d58f74fbace4b74dcf65481e734ed1714121bdc571da345540efa05"),
    },
];

pub struct AddressBookEntry {
    pub chain_id: FieldElement,
    pub name: &'static str,
    pub address: FieldElement,
}

/// A resolver that lazily fetches chain id to avoid unnecessary network calls.
pub struct AddressBookResolver<S> {
    chain_id_source: S,
    chain_id: OnceCell<FieldElement>,
}

impl<S> AddressBookResolver<S> {
    pub fn new(chain_id_source: S) -> Self {
        Self {
            chain_id_source,
            chain_id: OnceCell::new(),
        }
    }
}

impl<S> AddressBookResolver<S>
where
    S: ChainIdSource,
{
    pub async fn resolve_name(&self, name: &str) -> Result<Option<FieldElement>> {
        let chain_id_cell = &self.chain_id;

        let chain_id = match chain_id_cell.get() {
            Some(chain_id) => *chain_id,
            None => {
                let chain_id = self.chain_id_source.get_chain_id().await?;

                // It's OK if another thread set it first
                let _ = chain_id_cell.set(chain_id);

                chain_id
            }
        };

        Ok(HARDCODED_ADDRESS_BOOK.iter().find_map(|entry| {
            if entry.chain_id == chain_id && entry.name == name {
                Some(entry.address)
            } else {
                None
            }
        }))
    }
}
