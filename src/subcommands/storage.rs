use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use starknet::{core::types::BlockId, providers::Provider};

use crate::{
    address_book::AddressBookResolver, block_id::BlockIdParser, decode::FeltDecoder,
    verbosity::VerbosityArgs, ProviderArgs,
};

#[derive(Debug, Parser)]
pub struct Storage {
    #[clap(flatten)]
    provider: ProviderArgs,
    #[clap(
        long,
        value_parser = BlockIdParser,
        default_value = "pending",
        help = "Block number, hash, or tag (latest/pending)"
    )]
    block: BlockId,
    #[clap(help = "Contract address")]
    address: String,
    #[clap(help = "Storage key")]
    key: String,
    #[clap(flatten)]
    verbosity: VerbosityArgs,
}

impl Storage {
    pub async fn run(self) -> Result<()> {
        self.verbosity.setup_logging();

        let provider = Arc::new(self.provider.into_provider()?);
        let felt_decoder = FeltDecoder::new(AddressBookResolver::new(provider.clone()));

        let address = felt_decoder
            .decode_single_with_addr_fallback(&self.address)
            .await?;
        let key = felt_decoder
            .decode_single_with_storage_fallback(&self.key)
            .await?;

        let value = provider.get_storage_at(address, key, self.block).await?;

        println!("{:#064x}", value);

        Ok(())
    }
}
