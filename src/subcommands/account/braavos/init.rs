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
        AccountConfig, AccountVariant, BraavosAccountConfig, BraavosDeploymentContext,
        BraavosMultisigConfig, BraavosSigner, BraavosStarkSigner, DeploymentContext,
        DeploymentStatus, UndeployedStatus,
    },
    path::ExpandedPathbufParser,
    signer::SignerArgs,
};

/// Official hashes used as of extension version 3.37.4
const BRAAVOS_BASE_ACCOUNT_CLASS_HASH: FieldElement =
    felt!("0x013bfe114fb1cf405bfc3a7f8dbe2d91db146c17521d40dcf57e16d6b59fa8e6");
const BRAAVOS_ACCOUNT_CLASS_HASH: FieldElement =
    felt!("0x00816dd0297efc55dc1e7559020a3a825e81ef734b558f03c83325d4da7e6253");

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
            variant: AccountVariant::Braavos(BraavosAccountConfig {
                version: 1,
                implementation: None,
                multisig: BraavosMultisigConfig::Off,
                signers: vec![BraavosSigner::Stark(BraavosStarkSigner {
                    public_key: signer.get_public_key().await?.scalar(),
                })],
            }),
            deployment: DeploymentStatus::Undeployed(UndeployedStatus {
                class_hash: BRAAVOS_ACCOUNT_CLASS_HASH,
                salt,
                context: Some(DeploymentContext::Braavos(BraavosDeploymentContext {
                    base_account_class_hash: BRAAVOS_BASE_ACCOUNT_CLASS_HASH,
                })),
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
