use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use starknet::{
    core::{
        types::{BlockId, BlockTag, FieldElement, FunctionCall},
        utils::get_selector_from_name,
    },
    providers::Provider,
};

use crate::ProviderArgs;

#[derive(Debug, Parser)]
pub struct Call {
    #[clap(flatten)]
    provider: ProviderArgs,
    #[clap(help = "Contract address")]
    contract_address: FieldElement,
    #[clap(help = "Name of the function being called")]
    selector: String,
    #[clap(help = "Raw function call arguments")]
    args: Vec<FieldElement>,
}

impl Call {
    pub async fn run(self) -> Result<()> {
        let provider = Arc::new(self.provider.into_provider());

        let selector = get_selector_from_name(&self.selector)?;

        let result = provider
            .call(
                FunctionCall {
                    contract_address: self.contract_address,
                    entry_point_selector: selector,
                    calldata: self.args,
                },
                BlockId::Tag(BlockTag::Latest),
            )
            .await?;

        if result.is_empty() {
            println!("[]");
        } else {
            println!("[");

            for (ind_element, element) in result.iter().enumerate() {
                println!(
                    "    \"{:#064x}\"{}",
                    element,
                    if ind_element == result.len() - 1 {
                        ""
                    } else {
                        ","
                    }
                );
            }

            println!("]");
        }

        Ok(())
    }
}
