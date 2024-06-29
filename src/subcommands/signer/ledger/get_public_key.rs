use anyhow::Result;
use clap::Parser;
use starknet::signers::{ledger::LedgerStarknetApp, DerivationPath};

#[derive(Debug, Parser)]
pub struct GetPublicKey {
    #[clap(
        long,
        help = "Display the public key on Ledger's screen for extra security"
    )]
    display: bool,
    #[clap(help = "An HD wallet derivation path with EIP-2645 standard, such as \
        \"m/2645'/1195502025'/1470455285'/0'/0'/0\"")]
    path: DerivationPath,
}

impl GetPublicKey {
    pub async fn run(self) -> Result<()> {
        let ledger = LedgerStarknetApp::new().await?;

        if self.display {
            eprintln!("Please confirm the public key on your Ledger");
        }

        let public_key = ledger.get_public_key(self.path, self.display).await?;
        println!("{:#064x}", public_key.scalar());

        Ok(())
    }
}
