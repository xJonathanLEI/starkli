use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use starknet::signers::SigningKey;

#[derive(Debug, Parser)]
pub struct Inspect {
    #[clap(long, help = "Print the public key only")]
    raw: bool,
    #[clap(help = "Path to the JSON keystore")]
    file: PathBuf,
}

impl Inspect {
    pub fn run(self) -> Result<()> {
        if !self.file.exists() {
            anyhow::bail!("keystore file not found");
        }

        let password = rpassword::prompt_password("Enter password: ")?;

        let key = SigningKey::from_keystore(self.file, &password)?;

        if self.raw {
            println!("{:#064x}", key.verifying_key().scalar());
        } else {
            println!("Public key: {:#064x}", key.verifying_key().scalar());
        }

        Ok(())
    }
}
