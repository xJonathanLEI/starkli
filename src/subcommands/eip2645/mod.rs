use anyhow::Result;
use clap::{Parser, Subcommand};

mod echo;
use echo::Echo;

#[derive(Debug, Parser)]
pub struct Eip2645 {
    #[clap(subcommand)]
    command: Subcommands,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
    #[clap(about = "Print an EIP-2645 path in the universally accepted standard format")]
    Echo(Echo),
}

impl Eip2645 {
    pub fn run(self) -> Result<()> {
        match self.command {
            Subcommands::Echo(cmd) => cmd.run(),
        }
    }
}
