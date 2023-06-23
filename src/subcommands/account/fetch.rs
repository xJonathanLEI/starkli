use std::{io::Write, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use starknet::{
    core::types::{BlockId, BlockTag, FieldElement, FunctionCall},
    macros::selector,
    providers::Provider,
};

use crate::{
    account::{
        AccountConfig, AccountVariant, AccountVariantType, DeployedStatus, DeploymentStatus,
        OzAccountConfig, KNOWN_ACCOUNT_CLASSES,
    },
    ProviderArgs,
};

#[derive(Debug, Parser)]
pub struct Fetch {
    #[clap(flatten)]
    provider: ProviderArgs,
    #[clap(long, help = "Overwrite the file if it already exists")]
    force: bool,
    #[clap(long, help = "Path to save the account config file")]
    output: Option<PathBuf>,
    #[clap(help = "Contract address")]
    address: String,
}

impl Fetch {
    pub async fn run(self) -> Result<()> {
        // We allow not saving the config to just identify the account contract
        if self.output.is_none() {
            eprintln!(
                "{}",
                "NOTE: --output is not supplied and the account config won't be persisted."
                    .bright_magenta()
            );
        }

        if self.output.as_ref().is_some_and(|output| output.exists()) && !self.force {
            anyhow::bail!("account config file already exists");
        }

        let provider = self.provider.into_provider();
        let address = FieldElement::from_hex_be(&self.address)?;

        let class_hash = provider
            .get_class_hash_at(BlockId::Tag(BlockTag::Pending), address)
            .await?;

        let known_class = match KNOWN_ACCOUNT_CLASSES
            .iter()
            .find(|class| class.class_hash == class_hash)
        {
            Some(class) => class,
            None => {
                eprintln!(
                    "{} is not a known account class hash. \
                    If you believe this is a bug, submit a PR to:",
                    format!("{:#064x}", class_hash).bright_yellow()
                );
                eprintln!("    https://github.com/xJonathanLEI/starkli");
                anyhow::bail!("unknown class hash: {:#064x}", class_hash);
            }
        };

        eprintln!(
            "Account contract type identified as: {}",
            format!("{}", known_class.variant).bright_yellow()
        );
        eprintln!("Description: {}", known_class.description.bright_yellow());

        // No need to proceed if the user doesn't even want to save the config
        let output = match self.output {
            Some(output) => output,
            None => return Ok(()),
        };

        let account = match known_class.variant {
            AccountVariantType::OpenZeppelin => {
                let public_key = provider
                    .call(
                        FunctionCall {
                            contract_address: address,
                            entry_point_selector: selector!("getPublicKey"),
                            calldata: vec![],
                        },
                        BlockId::Tag(BlockTag::Pending),
                    )
                    .await?[0];

                AccountConfig {
                    version: 1,
                    variant: AccountVariant::OpenZeppelin(OzAccountConfig {
                        version: 1,
                        public_key,
                    }),
                    deployment: DeploymentStatus::Deployed(DeployedStatus {
                        class_hash,
                        address,
                    }),
                }
            }
        };

        let mut file = std::fs::File::create(&output)?;
        serde_json::to_writer_pretty(&mut file, &account)?;
        file.write_all(b"\n")?;

        eprintln!(
            "Downloaded new account config file: {}",
            std::fs::canonicalize(&output)?.display()
        );

        Ok(())
    }
}
