use anyhow::Result;
use clap::{Parser, Subcommand};

mod new;
use new::New;

mod inspect;
use inspect::Inspect;

mod inspect_private;
use inspect_private::InspectPrivate;

#[derive(Debug, Parser)]
pub struct Keystore {
    #[clap(subcommand)]
    command: Subcommands,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
    #[clap(about = "Randomly generate a new keystore")]
    New(New),
    #[clap(about = "Check the public key of an existing keystore file")]
    Inspect(Inspect),
    #[clap(about = "Check the private key of an existing keystore file")]
    InspectPrivate(InspectPrivate),
}

impl Keystore {
    pub fn run(self) -> Result<()> {
        match self.command {
            Subcommands::New(cmd) => cmd.run(),
            Subcommands::Inspect(cmd) => cmd.run(),
            Subcommands::InspectPrivate(cmd) => cmd.run(),
        }
    }
}
