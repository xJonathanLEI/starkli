use anyhow::Result;
use clap::Parser;
use colored_json::{ColorMode, Output};
use starknet::{
    core::types::{BlockId, BlockTag, FieldElement},
    providers::Provider,
};

use crate::ProviderArgs;

#[derive(Debug, Parser)]
pub struct ClassByHash {
    #[clap(flatten)]
    provider: ProviderArgs,
    #[clap(help = "Class hash")]
    hash: String,
}

impl ClassByHash {
    pub async fn run(self) -> Result<()> {
        let provider = self.provider.into_provider();
        let class_hash = FieldElement::from_hex_be(&self.hash)?;

        // TODO: allow custom block
        let class = provider
            .get_class(BlockId::Tag(BlockTag::Latest), class_hash)
            .await?;

        let class_json = serde_json::to_value(class)?;
        let class_json =
            colored_json::to_colored_json(&class_json, ColorMode::Auto(Output::StdOut))?;
        println!("{class_json}");

        Ok(())
    }
}
