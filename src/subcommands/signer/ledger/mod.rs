use anyhow::Result;
use clap::{Parser, Subcommand};

mod get_public_key;
use get_public_key::GetPublicKey;

#[derive(Debug, Parser)]
pub struct Ledger {
    #[clap(subcommand)]
    command: Subcommands,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
    #[clap(about = "Retrieve public key from a Ledger device")]
    GetPublicKey(GetPublicKey),
}

impl Ledger {
    pub async fn run(self) -> Result<()> {
        match self.command {
            Subcommands::GetPublicKey(cmd) => cmd.run().await,
        }
    }
}
