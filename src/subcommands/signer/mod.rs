use anyhow::Result;
use clap::{Parser, Subcommand};

mod keystore;
use keystore::Keystore;

#[cfg(feature = "ledger")]
pub mod ledger;
#[cfg(feature = "ledger")]
use ledger::Ledger;

mod gen_keypair;
use gen_keypair::GenKeypair;

#[derive(Debug, Parser)]
pub struct Signer {
    #[clap(subcommand)]
    command: Subcommands,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
    #[clap(about = "Keystore management commands")]
    Keystore(Keystore),
    #[cfg(feature = "ledger")]
    #[clap(about = "Ledger hardware wallet management commands")]
    Ledger(Ledger),
    #[clap(about = "Randomly generate a new key pair")]
    GenKeypair(GenKeypair),
}

impl Signer {
    pub async fn run(self) -> Result<()> {
        match self.command {
            Subcommands::Keystore(cmd) => cmd.run(),
            #[cfg(feature = "ledger")]
            Subcommands::Ledger(cmd) => cmd.run().await,
            Subcommands::GenKeypair(cmd) => cmd.run(),
        }
    }
}
