use anyhow::Result;
use clap::Parser;
use starknet::{
    core::types::{BlockId, BlockTag, ContractClass, Felt},
    providers::Provider,
};

use crate::{
    utils::{parse_compressed_legacy_class, parse_flattened_sierra_class, print_colored_json},
    verbosity::VerbosityArgs,
    ProviderArgs,
};

#[derive(Debug, Parser)]
pub struct ClassAt {
    #[clap(flatten)]
    provider: ProviderArgs,
    #[clap(
        long,
        help = "Attempt to recover a flattened Sierra class or a compressed legacy class"
    )]
    parse: bool,
    #[clap(help = "Contract address")]
    address: String,
    #[clap(flatten)]
    verbosity: VerbosityArgs,
}

impl ClassAt {
    pub async fn run(self) -> Result<()> {
        self.verbosity.setup_logging();

        let provider = self.provider.into_provider()?;
        let address = Felt::from_hex(&self.address)?;

        // TODO: allow custom block
        let class = provider
            .get_class_at(BlockId::Tag(BlockTag::Pending), address)
            .await?;

        if self.parse {
            match class {
                ContractClass::Sierra(class) => {
                    let class = parse_flattened_sierra_class(class)?;
                    print_colored_json(&class)?;
                }
                ContractClass::Legacy(class) => {
                    let class = parse_compressed_legacy_class(class)?;
                    print_colored_json(&class)?;
                }
            }
        } else {
            print_colored_json(&serde_json::to_value(class)?)?;
        }

        Ok(())
    }
}
