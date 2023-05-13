use anyhow::Result;
use clap::{Parser, Subcommand};

mod new;
use new::New;

#[derive(Debug, Parser)]
pub struct Keystore {
    #[clap(subcommand)]
    command: Subcommands,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
    #[clap(about = "Randomly generate a new keystore")]
    New(New),
}

impl Keystore {
    pub fn run(self) -> Result<()> {
        match self.command {
            Subcommands::New(cmd) => cmd.run(),
        }
    }
}
