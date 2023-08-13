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

/// Official hashes used as of extension version 3.21.10
const BRAAVOS_PROXY_CLASS_HASH: FieldElement =
    felt!("0x03131fa018d520a037686ce3efddeab8f28895662f019ca3ca18a626650f7d1e");
const BRAAVOS_MOCK_IMPL_CLASS_HASH: FieldElement =
    felt!("0x05aa23d5bb71ddaa783da7ea79d405315bafa7cf0387a74f4593578c3e9e6570");
const BRAAVOS_IMPL_CLASS_HASH: FieldElement =
    felt!("0x02c2b8f559e1221468140ad7b2352b1a5be32660d0bf1a3ae3a054a4ec5254e4");

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
                implementation: BRAAVOS_IMPL_CLASS_HASH,
                multisig: BraavosMultisigConfig::Off,
                signers: vec![BraavosSigner::Stark(BraavosStarkSigner {
                    public_key: signer.get_public_key().await?.scalar(),
                })],
            }),
            deployment: DeploymentStatus::Undeployed(UndeployedStatus {
                class_hash: BRAAVOS_PROXY_CLASS_HASH,
                salt,
                context: Some(DeploymentContext::Braavos(BraavosDeploymentContext {
                    mock_implementation: BRAAVOS_MOCK_IMPL_CLASS_HASH,
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
