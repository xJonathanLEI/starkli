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
        AccountConfig, AccountVariant, AccountVariantType, ArgentAccountConfig,
        BraavosMultisigConfig, BraavosSigner, DeployedStatus, DeploymentStatus, OzAccountConfig,
        KNOWN_ACCOUNT_CLASSES,
    },
    verbosity::VerbosityArgs,
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
    #[clap(flatten)]
    verbosity: VerbosityArgs,
}

impl Fetch {
    pub async fn run(self) -> Result<()> {
        self.verbosity.setup_logging();

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

        let variant = match known_class.variant {
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

                AccountVariant::OpenZeppelin(OzAccountConfig {
                    version: 1,
                    public_key,
                })
            }
            AccountVariantType::Argent => {
                let implementation = provider
                    .call(
                        FunctionCall {
                            contract_address: address,
                            entry_point_selector: selector!("get_implementation"),
                            calldata: vec![],
                        },
                        BlockId::Tag(BlockTag::Pending),
                    )
                    .await?[0];
                let signer = provider
                    .call(
                        FunctionCall {
                            contract_address: address,
                            entry_point_selector: selector!("getSigner"),
                            calldata: vec![],
                        },
                        BlockId::Tag(BlockTag::Pending),
                    )
                    .await?[0];
                let guardian = provider
                    .call(
                        FunctionCall {
                            contract_address: address,
                            entry_point_selector: selector!("getGuardian"),
                            calldata: vec![],
                        },
                        BlockId::Tag(BlockTag::Pending),
                    )
                    .await?[0];

                AccountVariant::Argent(ArgentAccountConfig {
                    version: 1,
                    implementation,
                    signer,
                    guardian,
                })
            }
            AccountVariantType::Braavos => {
                let implementation = provider
                    .call(
                        FunctionCall {
                            contract_address: address,
                            entry_point_selector: selector!("get_implementation"),
                            calldata: vec![],
                        },
                        BlockId::Tag(BlockTag::Pending),
                    )
                    .await?[0];
                let signers = provider
                    .call(
                        FunctionCall {
                            contract_address: address,
                            entry_point_selector: selector!("get_signers"),
                            calldata: vec![],
                        },
                        BlockId::Tag(BlockTag::Pending),
                    )
                    .await?;
                let multisig = provider
                    .call(
                        FunctionCall {
                            contract_address: address,
                            entry_point_selector: selector!("get_multisig"),
                            calldata: vec![],
                        },
                        BlockId::Tag(BlockTag::Pending),
                    )
                    .await?[0];

                let signers = {
                    let mut buffer = vec![];

                    let num_signers = TryInto::<u64>::try_into(signers[0])? as usize;

                    for ind_signer in 0..num_signers {
                        let base_offset = ind_signer * 8 + 1;

                        if Into::<FieldElement>::into(ind_signer as u64) != signers[base_offset] {
                            anyhow::bail!("unable to decode Braavos signers: index mismatch");
                        }

                        let signer =
                            BraavosSigner::decode(&signers[(base_offset + 1)..(base_offset + 8)])?;

                        buffer.push(signer);
                    }

                    buffer
                };

                let multisig = if multisig == FieldElement::ZERO {
                    BraavosMultisigConfig::Off
                } else {
                    BraavosMultisigConfig::On {
                        num_signers: TryInto::<u64>::try_into(multisig)? as usize,
                    }
                };

                AccountVariant::Braavos(crate::account::BraavosAccountConfig {
                    version: 1,
                    implementation,
                    multisig,
                    signers,
                })
            }
        };

        let account = AccountConfig {
            version: 1,
            variant,
            deployment: DeploymentStatus::Deployed(DeployedStatus {
                class_hash,
                address,
            }),
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
