use anyhow::Result;
use clap::{Parser, Subcommand};

mod fetch;
use fetch::Fetch;

mod deploy;
use deploy::Deploy;

mod oz;
use oz::Oz;

mod argent;
use argent::Argent;

#[derive(Debug, Parser)]
pub struct Account {
    #[clap(subcommand)]
    command: Subcommands,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
    #[clap(about = "Fetch account config from an already deployed account contract")]
    Fetch(Fetch),
    #[clap(about = "Deploy account contract with a DeployAccount transaction")]
    Deploy(Deploy),
    #[clap(about = "Create and manage OpenZeppelin account contracts")]
    Oz(Oz),
    #[clap(about = "Create and manage Argent X account contracts")]
    Argent(Argent),
}

impl Account {
    pub async fn run(self) -> Result<()> {
        match self.command {
            Subcommands::Fetch(cmd) => cmd.run().await,
            Subcommands::Deploy(cmd) => cmd.run().await,
            Subcommands::Oz(cmd) => cmd.run().await,
            Subcommands::Argent(cmd) => cmd.run().await,
        }
    }
}
