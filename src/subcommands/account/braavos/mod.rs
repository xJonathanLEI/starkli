use anyhow::Result;
use clap::{Parser, Subcommand};

mod init;
use init::Init;

#[derive(Debug, Parser)]
pub struct Braavos {
    #[clap(subcommand)]
    command: Subcommands,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
    #[clap(about = "Create a new account configuration without actually deploying")]
    Init(Init),
}

impl Braavos {
    pub async fn run(self) -> Result<()> {
        match self.command {
            Subcommands::Init(cmd) => cmd.run().await,
        }
    }
}
