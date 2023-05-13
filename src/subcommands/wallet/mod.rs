use anyhow::Result;
use clap::{Parser, Subcommand};

mod gen_keypair;
use gen_keypair::GenKeypair;

#[derive(Debug, Parser)]
pub struct Wallet {
    #[clap(subcommand)]
    command: Subcommands,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
    #[clap(about = "Randomly generate a new key pair")]
    GenKeypair(GenKeypair),
}

impl Wallet {
    pub fn run(self) -> Result<()> {
        match self.command {
            Subcommands::GenKeypair(cmd) => cmd.run(),
        }
    }
}
