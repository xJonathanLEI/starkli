use anyhow::Result;
use clap::Parser;
use starknet::core::utils::cairo_short_string_to_felt;

#[derive(Debug, Parser)]
pub struct ToCairoString {
    #[clap(long, help = "Display the encoded value in decimal representation")]
    dec: bool,
    #[clap(help = "Text to be encoded in felt")]
    text: String,
}

impl ToCairoString {
    pub fn run(self) -> Result<()> {
        let felt_value = cairo_short_string_to_felt(&self.text)?;
        if self.dec {
            println!("{}", felt_value);
        } else {
            println!("{:#x}", felt_value);
        }

        Ok(())
    }
}
