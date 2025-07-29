use anyhow::Result;
use clap::Parser;
use starknet::signers::ledger::LedgerStarknetApp;

#[derive(Debug, Parser)]
pub struct AppVersion;

impl AppVersion {
    pub async fn run(self) -> Result<()> {
        let ledger = LedgerStarknetApp::new().await?;

        let version = ledger.get_version().await?;
        println!("{version}");

        Ok(())
    }
}
