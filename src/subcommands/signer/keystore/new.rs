use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use starknet::signers::SigningKey;

use crate::path::ExpandedPathbufParser;

#[derive(Debug, Parser)]
pub struct New {
    #[clap(
        long,
        help = "Supply password from command line option instead of prompt"
    )]
    password: Option<String>,
    #[clap(long, help = "Overwrite the file if it already exists")]
    force: bool,
    #[clap(
        value_parser = ExpandedPathbufParser,
        help = "Path to save the JSON keystore"
    )]
    file: PathBuf,
}

impl New {
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

        let password = if let Some(password) = self.password {
            password
        } else {
            rpassword::prompt_password("Enter password: ")?
        };

        let key = SigningKey::from_random();
        key.save_as_keystore(&self.file, &password)?;

        println!(
            "Created new encrypted keystore file: {}",
            std::fs::canonicalize(self.file)?.display()
        );
        println!(
            "Public key: {}",
            format!("{:#064x}", key.verifying_key().scalar()).bright_yellow()
        );

        Ok(())
    }
}
