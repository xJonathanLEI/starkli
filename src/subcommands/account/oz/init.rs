use std::{io::Write, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use serde::Serialize;
use serde_with::serde_as;
use starknet::{
    core::{
        serde::unsigned_field_element::UfeHex, types::FieldElement, utils::get_contract_address,
    },
    macros::felt,
    signers::SigningKey,
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

#[derive(Serialize)]
struct AccountConfig {
    version: u64,
    variant: AccountVariant,
    deployment: DeploymentStatus,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum AccountVariant {
    OpenZeppelin(OzAccountConfig),
}

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum DeploymentStatus {
    Undeployed(UndeployedStatus),
}

#[serde_as]
#[derive(Serialize)]
struct OzAccountConfig {
    version: u64,
    #[serde_as(as = "UfeHex")]
    public_key: FieldElement,
}

#[serde_as]
#[derive(Serialize)]
struct UndeployedStatus {
    #[serde_as(as = "UfeHex")]
    class_hash: FieldElement,
    #[serde_as(as = "UfeHex")]
    salt: FieldElement,
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

        let deployed_address = get_contract_address(
            salt,
            OZ_ACCOUNT_CLASS_HASH,
            &[key.verifying_key().scalar()],
            FieldElement::ZERO,
        );

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
