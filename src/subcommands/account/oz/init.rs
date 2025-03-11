use std::{io::Write, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use starknet::{
    core::types::Felt,
    macros::felt,
    signers::{Signer, SigningKey},
};

use crate::{
    account::{AccountConfig, AccountVariant, DeploymentStatus, OzAccountConfig, UndeployedStatus},
    path::ExpandedPathbufParser,
    signer::SignerArgs,
};

/// OpenZeppelin account contract v1.0.0 compiled with cairo v2.9.4
const OZ_ACCOUNT_CLASS_HASH: Felt =
    felt!("0x05b4b537eaa2399e3aa99c4e2e0208ebd6c71bc1467938cd52c798c601e43564");

#[derive(Debug, Parser)]
pub struct Init {
    // TODO: allow manually specifying public key without using a signer
    #[clap(flatten)]
    signer: SignerArgs,
    #[clap(
        long,
        short,
        help = "Overwrite the account config file if it already exists"
    )]
    force: bool,
    #[clap(long, short, help = "Custom account class hash")]
    class_hash: Option<Felt>,
    #[clap(
        value_parser = ExpandedPathbufParser,
        help = "Path to save the account config file"
    )]
    output: PathBuf,
}

impl Init {
    pub async fn run(self) -> Result<()> {
        if self.output.exists() && !self.force {
            anyhow::bail!("account config file already exists");
        }

        let class_hash = match self.class_hash {
            Some(custom_hash) => {
                eprintln!(
                    "{}",
                    "WARNING: you're using a custom account class hash. \
                            The deployed account may not work as expected. \
                            Fetching custom accounts is currently not supported."
                        .bright_magenta()
                );

                custom_hash
            }
            None => OZ_ACCOUNT_CLASS_HASH,
        };

        let signer = self.signer.into_signer().await?;

        // Too lazy to write random salt generation
        let salt = SigningKey::from_random().secret_scalar();

        let account_config = AccountConfig {
            version: 1,
            variant: AccountVariant::OpenZeppelin(OzAccountConfig {
                version: 1,
                public_key: signer.get_public_key().await?.scalar(),
                legacy: false,
            }),
            deployment: DeploymentStatus::Undeployed(UndeployedStatus {
                class_hash,
                salt,
                context: None,
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
            format!("starkli account deploy {}", self.output.display()).bright_yellow()
        );

        Ok(())
    }
}
