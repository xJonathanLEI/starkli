use std::{io::Read, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use starknet::{core::types::FieldElement, signers::SigningKey};

use crate::path::ExpandedPathbufParser;

#[derive(Debug, Parser)]
pub struct FromKey {
    #[clap(long, help = "Overwrite the file if it already exists")]
    force: bool,
    #[clap(long, help = "Take the private key from stdin instead of prompt")]
    private_key_stdin: bool,
    #[clap(
        long,
        help = "Supply password from command line option instead of prompt"
    )]
    password: Option<String>,
    #[clap(
        value_parser = ExpandedPathbufParser,
        help = "Path to save the JSON keystore"
    )]
    file: PathBuf,
}

impl FromKey {
    pub fn run(self) -> Result<()> {
        if self.password.is_some() {
            eprintln!(
                "{}",
                "WARNING: setting passwords via --password is generally considered insecure, \
                as they will be stored in your shell history or other log files."
                    .bright_magenta()
            );
        }

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

        let password = if let Some(password) = self.password {
            password
        } else {
            rpassword::prompt_password("Enter password: ")?
        };

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
