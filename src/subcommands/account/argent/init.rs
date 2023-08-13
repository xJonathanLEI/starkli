use std::{io::Write, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use starknet::{
    core::types::FieldElement,
    macros::felt,
    signers::{Signer, SigningKey},
};

use crate::{
    account::{
        AccountConfig, AccountVariant, ArgentAccountConfig, DeploymentStatus, UndeployedStatus,
    },
    path::ExpandedPathbufParser,
    signer::SignerArgs,
};

/// Official hashes used as of extension version 5.7.0
const ARGENT_PROXY_CLASS_HASH: FieldElement =
    felt!("0x025ec026985a3bf9d0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a918");
const ARGENT_IMPL_CLASS_HASH: FieldElement =
    felt!("0x033434ad846cdd5f23eb73ff09fe6fddd568284a0fb7d1be20ee482f044dabe2");

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

        let signer = self.signer.into_signer()?;

        // Too lazy to write random salt generation
        let salt = SigningKey::from_random().secret_scalar();

        let account_config = AccountConfig {
            version: 1,
            variant: AccountVariant::Argent(ArgentAccountConfig {
                version: 1,
                implementation: ARGENT_IMPL_CLASS_HASH,
                signer: signer.get_public_key().await?.scalar(),
                guardian: FieldElement::ZERO,
            }),
            deployment: DeploymentStatus::Undeployed(UndeployedStatus {
                class_hash: ARGENT_PROXY_CLASS_HASH,
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
