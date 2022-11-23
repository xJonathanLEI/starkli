use anyhow::{anyhow, Result};
use clap::Parser;
use starknet::core::utils::get_selector_from_name;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Selector {
    #[clap(help = "Selector name")]
    name: String,
}

impl Selector {
    pub fn run(self) -> Result<()> {
        let trimmed_name = self.name.trim();

        if trimmed_name.contains('(') || trimmed_name.contains(')') {
            return Err(anyhow!(
                "parentheses and the content within should not be supplied"
            ));
        }

        let selector = get_selector_from_name(trimmed_name)?;
        println!("{:#064x}", selector);

        Ok(())
    }
}
