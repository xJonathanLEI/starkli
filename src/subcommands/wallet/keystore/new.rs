use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use starknet::signers::SigningKey;

#[derive(Debug, Parser)]
pub struct New {
    #[clap(long, help = "Overwrite the file if it already exists")]
    force: bool,
    #[clap(help = "Path to save the JSON keystore")]
    file: PathBuf,
}

impl New {
    pub fn run(self) -> Result<()> {
        if self.file.exists() && !self.force {
            anyhow::bail!("keystore file already exists");
        }

        let password = rpassword::prompt_password("Enter Password: ")?;

        let key = SigningKey::from_random();
        key.save_as_keystore(&self.file, &password)?;

        println!(
            "Created new encrypted keystore file: {}",
            std::fs::canonicalize(self.file)?.display()
        );
        println!("Public key: {:#064x}", key.verifying_key().scalar());

        Ok(())
    }
}
