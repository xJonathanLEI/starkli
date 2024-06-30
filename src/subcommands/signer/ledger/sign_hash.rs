use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use starknet::{
    core::types::Felt,
    signers::{ledger::LedgerStarknetApp, DerivationPath},
};

use crate::hd_path::DerivationPathParser;

#[derive(Debug, Parser)]
pub struct SignHash {
    #[clap(
        long,
        value_parser = DerivationPathParser,
        help = "An HD wallet derivation path with EIP-2645 standard, such as \
        \"m/2645'/starknet'/starkli'/0'/0'/0\""
    )]
    path: DerivationPath,
    #[clap(help = "The raw hash to be signed")]
    hash: String,
}

impl SignHash {
    pub async fn run(self) -> Result<()> {
        let hash = Felt::from_hex(&self.hash)?;

        eprintln!(
            "{}",
            "WARNING: blind signing a raw hash could be dangerous. Make sure you ONLY sign hashes \
            from trusted sources. If you're sending transactions, use Ledger as a signer instead \
            of using this command."
                .bright_magenta()
        );

        let ledger = LedgerStarknetApp::new().await?;

        eprintln!("Please confirm the signing operation on your Ledger");

        let signature = ledger.sign_hash(self.path, &hash).await?;
        println!("0x{}", signature);

        Ok(())
    }
}
