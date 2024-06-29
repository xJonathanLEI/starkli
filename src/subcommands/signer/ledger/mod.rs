use anyhow::Result;
use clap::{Parser, Subcommand};

mod get_public_key;
use get_public_key::GetPublicKey;

mod sign_hash;
use sign_hash::SignHash;

mod app_version;
use app_version::AppVersion;

#[derive(Debug, Parser)]
pub struct Ledger {
    #[clap(subcommand)]
    command: Subcommands,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
    #[clap(about = "Retrieve public key from a Ledger device")]
    GetPublicKey(GetPublicKey),
    #[clap(about = "Blind sign a raw hash with a Ledger device")]
    SignHash(SignHash),
    #[clap(about = "Retrieve the Starknet app version on a Ledger device")]
    AppVersion(AppVersion),
}

impl Ledger {
    pub async fn run(self) -> Result<()> {
        match self.command {
            Subcommands::GetPublicKey(cmd) => cmd.run().await,
            Subcommands::SignHash(cmd) => cmd.run().await,
            Subcommands::AppVersion(cmd) => cmd.run().await,
        }
    }
}
