use anyhow::Result;
use clap::Parser;
use starknet::{core::utils::parse_cairo_short_string, providers::Provider};

use crate::{verbosity::VerbosityArgs, ProviderArgs};

#[derive(Debug, Parser)]
pub struct ChainId {
    #[clap(flatten)]
    provider: ProviderArgs,
    #[clap(long, help = "Do not show the decoded text")]
    no_decode: bool,
    #[clap(
        long,
        help = "Display the decimal instead of hexadecimal representation"
    )]
    dec: bool,
    #[clap(
        default_value = "latest",
        help = "Block number, hash, or tag (latest/pending)"
    )]
    block_id: String,
    #[clap(flatten)]
    verbosity: VerbosityArgs,
}

impl ChainId {
    pub async fn run(self) -> Result<()> {
        self.verbosity.setup_logging();

        let provider = self.provider.into_provider()?;

        let raw_chain_id = provider.chain_id().await?;

        println!(
            "{}{}",
            if self.dec {
                format!("{raw_chain_id}")
            } else {
                format!("{raw_chain_id:#x}")
            },
            if self.no_decode {
                "".into()
            } else {
                let decoded = parse_cairo_short_string(&raw_chain_id)?;
                format!(" ({decoded})")
            }
        );

        Ok(())
    }
}
