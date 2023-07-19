use anyhow::Result;
use clap::{Parser, Subcommand};

mod mine_udc_salt;
use mine_udc_salt::MineUdcSalt;

#[derive(Debug, Parser)]
pub struct Lab {
    #[clap(subcommand)]
    command: Subcommands,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
    #[clap(about = "Mine UDC contract deployment salt for specific address prefix and/or suffix")]
    MineUdcSalt(MineUdcSalt),
}

impl Lab {
    pub fn run(self) -> Result<()> {
        match self.command {
            Subcommands::MineUdcSalt(cmd) => cmd.run(),
        }
    }
}
