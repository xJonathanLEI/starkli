use anyhow::Result;
use clap::Parser;

use crate::utils::parse_felt_value;

#[derive(Debug, Parser)]
pub struct Mont {
    #[clap(long, help = "Emit array elements in hexadecimal format")]
    hex: bool,
    #[clap(help = "Encoded string value in felt, in decimal or hexadecimal representation")]
    felt: String,
}

impl Mont {
    pub fn run(self) -> Result<()> {
        let felt = parse_felt_value(&self.felt)?;
        let mont = felt.to_raw_reversed();

        let mut output = String::new();

        output.push_str("[\n");

        for element in mont.into_iter() {
            output.push_str(&if self.hex {
                format!("    {element:#x},\n")
            } else {
                format!("    {element},\n")
            });
        }

        output.push_str("]\n");

        print!("{output}");

        Ok(())
    }
}
