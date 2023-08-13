use anyhow::Result;
use clap::{Parser, Subcommand};

mod init;
use init::Init;

#[derive(Debug, Parser)]
pub struct Argent {
    #[clap(subcommand)]
    command: Subcommands,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
    #[clap(about = "Create a new account configuration without actually deploying")]
    Init(Init),
}

impl Argent {
    pub async fn run(self) -> Result<()> {
        match self.command {
            Subcommands::Init(cmd) => cmd.run().await,
        }
    }
}
