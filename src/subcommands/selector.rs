use anyhow::Result;
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
        let selector = get_selector_from_name(self.name.trim())?;
        println!("{:#064x}", selector);

        Ok(())
    }
}
