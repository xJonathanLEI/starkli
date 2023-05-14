use anyhow::Result;
use clap::{Parser, Subcommand};

mod init;
use init::Init;

mod deploy;
use deploy::Deploy;

#[derive(Debug, Parser)]
pub struct Oz {
    #[clap(subcommand)]
    command: Subcommands,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
    #[clap(about = "Create a new account configuration without actually deploying")]
    Init(Init),
    #[clap(about = "Deploy account contract with a DeployAccount transaction")]
    Deploy(Deploy),
}

impl Oz {
    pub async fn run(self) -> Result<()> {
        match self.command {
            Subcommands::Init(cmd) => cmd.run(),
            Subcommands::Deploy(cmd) => cmd.run().await,
        }
    }
}
