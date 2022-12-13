use anyhow::Result;
use clap::Parser;
use starknet::core::utils::parse_cairo_short_string;

use crate::utils::parse_felt_value;

#[derive(Debug, Parser)]
pub struct ParseCairoString {
    #[clap(help = "Encoded string value in felt, in decimal or hexadecimal representation")]
    felt: String,
}

impl ParseCairoString {
    pub fn run(self) -> Result<()> {
        let felt = parse_felt_value(&self.felt)?;
        let decoded = parse_cairo_short_string(&felt)?;
        println!("{decoded}");

        Ok(())
    }
}
