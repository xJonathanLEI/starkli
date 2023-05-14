use std::{io::Write, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use starknet::{core::types::FieldElement, macros::felt, signers::SigningKey};

use crate::account::{
    AccountConfig, AccountVariant, DeploymentStatus, OzAccountConfig, UndeployedStatus,
};

/// OpenZeppelin account contract v0.6.1 compiled with cairo-lang v0.11.0.2
const OZ_ACCOUNT_CLASS_HASH: FieldElement =
    felt!("0x048dd59fabc729a5db3afdf649ecaf388e931647ab2f53ca3c6183fa480aa292");

#[derive(Debug, Parser)]
pub struct Init {
    // TODO: allow manually specifying public key without using a wallet
    #[clap(long, help = "Path to keystore JSON file for reading the public key")]
    keystore: PathBuf,
    #[clap(
        long,
        short,
        help = "Overwrite the account config file if it already exists"
    )]
    force: bool,
    #[clap(help = "Path to save the account config file")]
    output: PathBuf,
}

impl Init {
    pub fn run(self) -> Result<()> {
        if self.output.exists() && !self.force {
            anyhow::bail!("account config file already exists");
        }

        if !self.keystore.exists() {
            anyhow::bail!("keystore file not found");
        }

        let password = rpassword::prompt_password("Enter keystore password: ")?;
        let key = SigningKey::from_keystore(self.keystore, &password)?;

        // Too lazy to write random salt generation
        let salt = SigningKey::from_random().secret_scalar();

        let account_config = AccountConfig {
            version: 1,
            variant: AccountVariant::OpenZeppelin(OzAccountConfig {
                version: 1,
                public_key: key.verifying_key().scalar(),
            }),
            deployment: DeploymentStatus::Undeployed(UndeployedStatus {
                class_hash: OZ_ACCOUNT_CLASS_HASH,
                salt,
            }),
        };

        let deployed_address = account_config.deploy_account_address()?;

        let mut file = std::fs::File::create(&self.output)?;
        serde_json::to_writer_pretty(&mut file, &account_config)?;
        file.write_all(b"\n")?;

        eprintln!(
            "Created new account config file: {}",
            std::fs::canonicalize(&self.output)?.display()
        );
        eprintln!();
        eprintln!(
            "Once deployed, this account will be available at:\n    {}",
            format!("{:#064x}", deployed_address).bright_yellow()
        );
        eprintln!();
        eprintln!(
            "Deploy this account by running:\n    {}",
            format!("starkli account oz deploy {}", self.output.display()).bright_yellow()
        );

        Ok(())
    }
}
