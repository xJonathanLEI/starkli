use anyhow::Result;
use clap::Parser;
use colored_json::{ColorMode, Output};
use starknet::providers::Provider;

use crate::{utils::parse_block_id, ProviderArgs};

#[derive(Debug, Parser)]
pub struct StateUpdate {
    #[clap(flatten)]
    provider: ProviderArgs,
    #[clap(
        default_value = "latest",
        help = "Block number, hash, or tag (latest/pending)"
    )]
    block_id: String,
}

impl StateUpdate {
    pub async fn run(self) -> Result<()> {
        let provider = self.provider.into_provider();

        let block_id = parse_block_id(&self.block_id)?;

        let update_json = serde_json::to_value(provider.get_state_update(block_id).await?)?;

        let update_json =
            colored_json::to_colored_json(&update_json, ColorMode::Auto(Output::StdOut))?;
        println!("{update_json}");

        Ok(())
    }
}
