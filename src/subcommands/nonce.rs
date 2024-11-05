use anyhow::Result;
use clap::Parser;
use starknet::{
    core::types::{BlockId, BlockTag, Felt},
    providers::Provider,
};

use crate::{provider::ProviderArgs, verbosity::VerbosityArgs};

#[derive(Debug, Parser)]
pub struct Nonce {
    #[clap(flatten)]
    provider: ProviderArgs,
    #[clap(help = "Contract address")]
    address: String,
    #[clap(flatten)]
    verbosity: VerbosityArgs,
}

impl Nonce {
    pub async fn run(self) -> Result<()> {
        self.verbosity.setup_logging();

        let provider = self.provider.into_provider()?;
        let address = Felt::from_hex(&self.address)?;

        // TODO: allow custom block
        let nonce = provider
            .get_nonce(BlockId::Tag(BlockTag::Pending), address)
            .await?;

        println!("{}", nonce);

        Ok(())
    }
}
