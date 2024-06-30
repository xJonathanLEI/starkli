use anyhow::Result;
use clap::Parser;

use crate::hd_path::{Eip2645Path, Eip2645PathParser};

#[derive(Debug, Parser)]
pub struct Echo {
    #[clap(
        value_parser = Eip2645PathParser,
        help = "An HD wallet derivation path with EIP-2645 standard, such as \
        \"m/2645'/starknet'/starkli'/0'/0'/0\""
    )]
    path: Eip2645Path,
}

impl Echo {
    pub fn run(self) -> Result<()> {
        println!("{}", self.path);

        Ok(())
    }
}
