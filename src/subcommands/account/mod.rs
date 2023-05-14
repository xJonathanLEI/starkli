use anyhow::Result;
use clap::{Parser, Subcommand};

mod oz;
use oz::Oz;

#[derive(Debug, Parser)]
pub struct Account {
    #[clap(subcommand)]
    command: Subcommands,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
    #[clap(about = "Create, deploy, and manage OpenZeppelin account contracts")]
    Oz(Oz),
}

impl Account {
    pub fn run(self) -> Result<()> {
        match self.command {
            Subcommands::Oz(cmd) => cmd.run(),
        }
    }
}
