use anyhow::Result;
use clap::Parser;
use starknet::signers::{ledger::LedgerStarknetApp, DerivationPath};

use crate::hd_path::DerivationPathParser;

#[derive(Debug, Parser)]
pub struct GetPublicKey {
    #[clap(
        long,
        help = "Do not display the public key on Ledger's screen for confirmation"
    )]
    no_display: bool,
    #[clap(
        value_parser = DerivationPathParser,
        help = "An HD wallet derivation path with EIP-2645 standard, such as \
        \"m/2645'/starknet'/starkli'/0'/0'/0\""
    )]
    path: DerivationPath,
}

impl GetPublicKey {
    pub async fn run(self) -> Result<()> {
        let ledger = LedgerStarknetApp::new().await?;

        if !self.no_display {
            eprintln!("Please confirm the public key on your Ledger");
        }

        let public_key = ledger.get_public_key(self.path, !self.no_display).await?;
        println!("{:#064x}", public_key.scalar());

        Ok(())
    }
}
