use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use starknet::{core::types::FieldElement, signers::SigningKey};

#[derive(Debug, Parser)]
pub struct FromKey {
    #[clap(long, help = "Overwrite the file if it already exists")]
    force: bool,
    #[clap(help = "Path to save the JSON keystore")]
    file: PathBuf,
}

impl FromKey {
    pub fn run(self) -> Result<()> {
        if self.file.exists() && !self.force {
            anyhow::bail!("keystore file already exists");
        }

        let private_key =
            FieldElement::from_hex_be(&rpassword::prompt_password("Enter private key: ")?)?;
        let password = rpassword::prompt_password("Enter password: ")?;

        let key = SigningKey::from_secret_scalar(private_key);
        key.save_as_keystore(&self.file, &password)?;

        println!(
            "Created new encrypted keystore file: {}",
            std::fs::canonicalize(self.file)?.display()
        );
        println!("Public key: {:#064x}", key.verifying_key().scalar());

        Ok(())
    }
}
