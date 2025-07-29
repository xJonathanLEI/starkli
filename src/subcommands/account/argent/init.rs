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
    account::{
        AccountConfig, AccountVariant, ArgentAccountConfig, DeploymentStatus, UndeployedStatus,
    },
    path::ExpandedPathbufParser,
    signer::SignerArgs,
};

/// Official hashes used as of extension version 5.16.3 compiled with cairo v2.6.3
const ARGENT_CLASS_HASH: Felt =
    felt!("0x036078334509b514626504edc9fb252328d1a240e4e948bef8d0c08dff45927f");

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

        let signer = self.signer.into_signer().await?;

        // Too lazy to write random salt generation
        let salt = SigningKey::from_random().secret_scalar();

        let account_config = AccountConfig {
            version: 1,
            variant: AccountVariant::Argent(ArgentAccountConfig {
                version: 1,
                implementation: None,
                owner: signer.get_public_key().await?.scalar(),
                guardian: Felt::ZERO,
            }),
            deployment: DeploymentStatus::Undeployed(UndeployedStatus {
                class_hash: ARGENT_CLASS_HASH,
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
            format!("{deployed_address:#064x}").bright_yellow()
        );
        eprintln!();
        eprintln!(
            "Deploy this account by running:\n    {}",
            format!("starkli account deploy {}", self.output.display()).bright_yellow()
        );

        Ok(())
    }
}
