use std::{io::Read, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use starknet::{core::types::FieldElement, signers::SigningKey};

#[derive(Debug, Parser)]
pub struct FromKey {
    #[clap(long, help = "Overwrite the file if it already exists")]
    force: bool,
    #[clap(long, help = "Take the private key from stdin instead of prompt")]
    private_key_stdin: bool,
    #[clap(help = "Path to save the JSON keystore")]
    file: PathBuf,
}

impl FromKey {
    pub fn run(self) -> Result<()> {
        if self.file.exists() && !self.force {
            anyhow::bail!("keystore file already exists");
        }

        let private_key = if self.private_key_stdin {
            let mut buffer = String::new();
            std::io::stdin().read_to_string(&mut buffer)?;

            buffer
        } else {
            rpassword::prompt_password("Enter private key: ")?
        };
        let private_key = FieldElement::from_hex_be(private_key.trim())?;

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
