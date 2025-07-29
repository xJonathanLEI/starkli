use std::{io::Write, path::PathBuf, sync::Arc, time::Duration};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use starknet::{
    accounts::{
        AccountDeploymentV3, AccountFactory, ArgentAccountFactory, OpenZeppelinAccountFactory,
    },
    core::types::{BlockId, BlockTag, FeeEstimate, Felt, NonZeroFelt},
    providers::Provider,
    signers::Signer,
};

use crate::{
    account::{
        AccountConfig, AccountVariant, BraavosMultisigConfig, BraavosSigner, DeployedStatus,
        DeploymentContext, DeploymentStatus,
    },
    account_factory::{AnyAccountFactory, BraavosAccountFactory},
    error::account_factory_error_mapper,
    fee::{FeeArgs, FeeSetting, FeeToken, TokenFeeSetting},
    path::ExpandedPathbufParser,
    provider::ExtendedProvider,
    signer::{AnySigner, SignerArgs},
    utils::{felt_to_bigdecimal, is_affected_braavos_class, print_colored_json, watch_tx},
    verbosity::VerbosityArgs,
    ProviderArgs,
};

#[derive(Debug, Parser)]
pub struct Deploy {
    #[clap(flatten)]
    provider: ProviderArgs,
    #[clap(flatten)]
    signer: SignerArgs,
    #[clap(flatten)]
    fee: FeeArgs,
    #[clap(long, help = "Simulate the transaction only")]
    simulate: bool,
    #[clap(long, help = "Provide transaction nonce manually")]
    nonce: Option<Felt>,
    #[clap(
        long,
        env = "STARKNET_POLL_INTERVAL",
        default_value = "5000",
        help = "Transaction result poll interval in milliseconds"
    )]
    poll_interval: u64,
    #[clap(
        value_parser = ExpandedPathbufParser,
        help = "Path to the account config file"
    )]
    file: PathBuf,
    #[clap(flatten)]
    verbosity: VerbosityArgs,
}

#[derive(Debug, Clone, Copy)]
enum MaxFeeType {
    Manual {
        max_fee: Felt,
    },
    Estimated {
        estimate: Felt,
        estimate_with_buffer: Felt,
    },
}

struct AmountWithBuffer<T> {
    amount: T,
    amount_with_buffer: T,
}

impl Deploy {
    pub async fn run(self) -> Result<()> {
        self.verbosity.setup_logging();

        let fee_setting = self.fee.into_setting()?;
        if self.simulate && fee_setting.is_estimate_only() {
            anyhow::bail!("--simulate cannot be used with --estimate-only");
        }

        let provider = Arc::new(self.provider.into_provider()?);
        let signer = Arc::new(self.signer.into_signer().await?);

        if !self.file.exists() {
            anyhow::bail!("account config file not found");
        }

        let mut account: AccountConfig =
            serde_json::from_reader(&mut std::fs::File::open(&self.file)?)?;

        if is_affected_braavos_class(account.deployment.class_hash()) {
            eprintln!(
                "{}",
                "WARNING: This Braavos account contract does not work with JSON-RPC \
                v0.8.x. Transactions WILL fail."
                    .bright_magenta()
            );
        }

        let signer_public_key = signer.get_public_key().await?.scalar();

        let undeployed_status = match &account.deployment {
            DeploymentStatus::Undeployed(inner) => inner,
            DeploymentStatus::Deployed(_) => {
                anyhow::bail!("account already deployed");
            }
        };

        let chain_id = provider.chain_id().await?;

        let factory = match &account.variant {
            AccountVariant::OpenZeppelin(oz_config) => {
                // Makes sure we're using the right key
                if signer_public_key != oz_config.public_key {
                    anyhow::bail!(
                        "public key mismatch. Expected: {:#064x}; actual: {:#064x}.",
                        oz_config.public_key,
                        signer_public_key
                    );
                }

                let mut factory = OpenZeppelinAccountFactory::new(
                    undeployed_status.class_hash,
                    chain_id,
                    signer.clone(),
                    provider.clone(),
                )
                .await?;
                factory.set_block_id(BlockId::Tag(BlockTag::Pending));

                AnyAccountFactory::OpenZeppelin(factory)
            }
            AccountVariant::Argent(argent_config) => {
                // It's probably not worth it to continue to support legacy account deployment.
                // Users can always deploy with an old Starkli version.
                if argent_config.implementation.is_some() {
                    anyhow::bail!(
                        "deployment of legacy Argent X (Cairo 0) accounts is no longer supported"
                    );
                }

                // Makes sure we're using the right key
                if signer_public_key != argent_config.owner {
                    anyhow::bail!(
                        "public key mismatch. Expected: {:#064x}; actual: {:#064x}.",
                        argent_config.owner,
                        signer_public_key
                    );
                }

                let mut factory = ArgentAccountFactory::new(
                    undeployed_status.class_hash,
                    chain_id,
                    None,
                    signer.clone(),
                    provider.clone(),
                )
                .await?;
                factory.set_block_id(BlockId::Tag(BlockTag::Pending));

                AnyAccountFactory::Argent(factory)
            }
            AccountVariant::Braavos(braavos_config) => {
                if !matches!(braavos_config.multisig, BraavosMultisigConfig::Off) {
                    anyhow::bail!("Braavos accounts cannot be deployed with multisig on");
                }
                if braavos_config.signers.len() != 1 {
                    anyhow::bail!("Braavos accounts can only be deployed with one seed signer");
                }

                match &undeployed_status.context {
                    Some(DeploymentContext::Braavos(context)) => {
                        // Safe to unwrap as we already checked for length
                        match braavos_config.signers.first().unwrap() {
                            BraavosSigner::Stark(stark_signer) => {
                                // Makes sure we're using the right key
                                if signer_public_key != stark_signer.public_key {
                                    anyhow::bail!(
                                        "public key mismatch. \
                                        Expected: {:#064x}; actual: {:#064x}.",
                                        stark_signer.public_key,
                                        signer_public_key
                                    );
                                }

                                let mut factory = BraavosAccountFactory::new(
                                    undeployed_status.class_hash,
                                    context.base_account_class_hash,
                                    chain_id,
                                    signer.clone(),
                                    provider.clone(),
                                )
                                .await?;
                                factory.set_block_id(BlockId::Tag(BlockTag::Pending));

                                AnyAccountFactory::Braavos(factory)
                            } // Reject other variants as we add more types
                        }
                    }
                    _ => anyhow::bail!("missing Braavos deployment context"),
                }
            }
        };

        let target_deployment_address = account.deploy_account_address()?;

        let account_deployment_tx = match fee_setting {
            FeeSetting::Strk(fee_setting) => {
                let account_deployment = factory.deploy_v3(undeployed_status.salt);
                let account_deployment = match self.nonce {
                    Some(nonce) => account_deployment.nonce(nonce),
                    None => account_deployment,
                };

                // Sanity check. We don't really need to check again here actually
                if account_deployment.address() != target_deployment_address {
                    panic!("Unexpected account deployment address mismatch");
                }

                let (fee_type, account_deployment) = match fee_setting {
                    TokenFeeSetting::Manual(fee) => match (
                        fee.l1_gas,
                        fee.l1_gas_price,
                        fee.l2_gas,
                        fee.l2_gas_price,
                        fee.l1_data_gas,
                        fee.l1_data_gas_price,
                    ) {
                        // Fees fully specified
                        (
                            Some(l1_gas),
                            Some(l1_gas_price),
                            Some(l2_gas),
                            Some(l2_gas_price),
                            Some(l1_data_gas),
                            Some(l1_data_gas_price),
                        ) => (
                            MaxFeeType::Manual {
                                max_fee: Felt::from(l1_gas) * Felt::from(l1_gas_price)
                                    + Felt::from(l2_gas) * Felt::from(l2_gas_price)
                                    + Felt::from(l1_data_gas) * Felt::from(l1_data_gas_price),
                            },
                            account_deployment
                                .l1_gas(l1_gas)
                                .l1_gas_price(l1_gas_price)
                                .l2_gas(l2_gas)
                                .l2_gas_price(l2_gas_price)
                                .l1_data_gas(l1_data_gas)
                                .l1_data_gas_price(l1_data_gas_price),
                        ),
                        // All gas amounts specified: just need to find gas price
                        (
                            Some(l1_gas),
                            l1_gas_price,
                            Some(l2_gas),
                            l2_gas_price,
                            Some(l1_data_gas),
                            l1_data_gas_price,
                        ) => {
                            let block = provider
                                .get_block_with_tx_hashes(factory.block_id())
                                .await?;

                            let l1_gas_price = resolve_amount_buffer(
                                &l1_gas_price,
                                block.l1_gas_price().price_in_fri,
                            )?;
                            let l2_gas_price = resolve_amount_buffer(
                                &l2_gas_price,
                                block.l2_gas_price().price_in_fri,
                            )?;
                            let l1_data_gas_price = resolve_amount_buffer(
                                &l1_data_gas_price,
                                block.l1_data_gas_price().price_in_fri,
                            )?;

                            (
                                MaxFeeType::Estimated {
                                    estimate: Felt::from(l1_gas) * Felt::from(l1_gas_price.amount)
                                        + Felt::from(l2_gas) * Felt::from(l2_gas_price.amount)
                                        + Felt::from(l1_data_gas)
                                            * Felt::from(l1_data_gas_price.amount),
                                    estimate_with_buffer: Felt::from(l1_gas)
                                        * Felt::from(l1_gas_price.amount_with_buffer)
                                        + Felt::from(l2_gas)
                                            * Felt::from(l2_gas_price.amount_with_buffer)
                                        + Felt::from(l1_data_gas)
                                            * Felt::from(l1_data_gas_price.amount_with_buffer),
                                },
                                account_deployment
                                    .l1_gas(l1_gas)
                                    .l1_gas_price(l1_gas_price.amount_with_buffer)
                                    .l2_gas(l2_gas)
                                    .l2_gas_price(l2_gas_price.amount_with_buffer)
                                    .l1_data_gas(l1_data_gas)
                                    .l1_data_gas_price(l1_data_gas_price.amount_with_buffer),
                            )
                        }
                        // Full estimation needed
                        (
                            l1_gas,
                            l1_gas_price,
                            l2_gas,
                            l2_gas_price,
                            l1_data_gas,
                            l1_data_gas_price,
                        ) => {
                            let estimated_fee = account_deployment
                                .estimate_fee()
                                .await
                                .map_err(account_factory_error_mapper)?;

                            resolve_estimate_buffer(
                                account_deployment,
                                &estimated_fee,
                                &l1_gas,
                                &l1_gas_price,
                                &l2_gas,
                                &l2_gas_price,
                                &l1_data_gas,
                                &l1_data_gas_price,
                            )?
                        }
                    },
                    TokenFeeSetting::EstimateOnly | TokenFeeSetting::None => {
                        let estimated_fee = account_deployment
                            .estimate_fee()
                            .await
                            .map_err(account_factory_error_mapper)?;

                        if fee_setting.is_estimate_only() {
                            println!(
                                "{} STRK",
                                format!("{}", felt_to_bigdecimal(estimated_fee.overall_fee, 18))
                                    .bright_yellow(),
                            );
                            return Ok(());
                        }

                        resolve_estimate_buffer(
                            account_deployment,
                            &estimated_fee,
                            &None,
                            &None,
                            &None,
                            &None,
                            &None,
                            &None,
                        )?
                    }
                };

                if self.simulate {
                    print_colored_json(&account_deployment.simulate(false, false).await?)?;
                    return Ok(());
                }

                fee_prompt(fee_type, target_deployment_address, FeeToken::Strk)?;

                account_deployment.send().await
            }
        }
        .map_err(account_factory_error_mapper)?
        .transaction_hash;

        eprintln!(
            "Account deployment transaction: {}",
            format!("{account_deployment_tx:#064x}").bright_yellow()
        );

        // By default we wait for the tx to confirm so that we don't incorrectly mark the account
        // as deployed
        eprintln!(
            "Waiting for transaction {} to confirm. \
            If this process is interrupted, you will need to run `{}` to update the account file.",
            format!("{account_deployment_tx:#064x}").bright_yellow(),
            "starkli account fetch".bright_yellow(),
        );
        watch_tx(
            &provider,
            account_deployment_tx,
            Duration::from_millis(self.poll_interval),
        )
        .await?;

        account.deployment = DeploymentStatus::Deployed(DeployedStatus {
            class_hash: undeployed_status.class_hash,
            address: target_deployment_address,
        });

        // Never write directly to the original file to avoid data loss
        let mut temp_file_name = self
            .file
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("unable to determine file name"))?
            .to_owned();
        temp_file_name.push(".tmp");
        let mut temp_path = self.file.clone();
        temp_path.set_file_name(temp_file_name);

        let mut temp_file = std::fs::File::create(&temp_path)?;
        serde_json::to_writer_pretty(&mut temp_file, &account)?;
        temp_file.write_all(b"\n")?;
        std::fs::rename(temp_path, self.file)?;

        Ok(())
    }
}

fn fee_prompt(fee_type: MaxFeeType, deployed_address: Felt, fee_token: FeeToken) -> Result<()> {
    match fee_type {
        MaxFeeType::Manual { max_fee } => {
            eprintln!(
                "You've manually specified the account deployment fee to be {}. \
                Therefore, fund at least:\n    {}",
                format!("{} {}", felt_to_bigdecimal(max_fee, 18), fee_token).bright_yellow(),
                format!("{} {}", felt_to_bigdecimal(max_fee, 18), fee_token).bright_yellow(),
            );
        }
        MaxFeeType::Estimated {
            estimate,
            estimate_with_buffer,
        } => {
            eprintln!(
                "The estimated account deployment fee is {}. \
                However, to avoid failure, fund at least:\n    {}",
                format!("{} {}", felt_to_bigdecimal(estimate, 18), fee_token).bright_yellow(),
                format!(
                    "{} {}",
                    felt_to_bigdecimal(estimate_with_buffer, 18),
                    fee_token
                )
                .bright_yellow()
            );
        }
    }

    eprintln!(
        "to the following address:\n    {}",
        format!("{deployed_address:#064x}").bright_yellow()
    );

    // TODO: add flag for skipping this manual confirmation step
    eprint!("Press [ENTER] once you've funded the address.");
    std::io::stdin().read_line(&mut String::new())?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
fn resolve_estimate_buffer<'a>(
    account_deployment: AccountDeploymentV3<
        'a,
        AnyAccountFactory<Arc<AnySigner>, Arc<ExtendedProvider>>,
    >,
    estimated_fee: &FeeEstimate,
    l1_gas: &Option<u64>,
    l1_gas_price: &Option<u128>,
    l2_gas: &Option<u64>,
    l2_gas_price: &Option<u128>,
    l1_data_gas: &Option<u64>,
    l1_data_gas_price: &Option<u128>,
) -> Result<(
    MaxFeeType,
    AccountDeploymentV3<'a, AnyAccountFactory<Arc<AnySigner>, Arc<ExtendedProvider>>>,
)> {
    let l1_gas = resolve_amount_buffer(l1_gas, estimated_fee.l1_gas_consumed)?;
    let l2_gas = resolve_amount_buffer(l2_gas, estimated_fee.l2_gas_consumed)?;
    let l1_data_gas = resolve_amount_buffer(l1_data_gas, estimated_fee.l1_data_gas_consumed)?;

    let l1_gas_price = resolve_amount_buffer(l1_gas_price, estimated_fee.l1_gas_price)?;
    let l2_gas_price = resolve_amount_buffer(l2_gas_price, estimated_fee.l2_gas_price)?;
    let l1_data_gas_price =
        resolve_amount_buffer(l1_data_gas_price, estimated_fee.l1_data_gas_price)?;

    Ok((
        MaxFeeType::Estimated {
            estimate: Felt::from(l1_gas.amount) * Felt::from(l1_gas_price.amount)
                + Felt::from(l2_gas.amount) * Felt::from(l2_gas_price.amount)
                + Felt::from(l1_data_gas.amount) * Felt::from(l1_data_gas_price.amount),
            estimate_with_buffer: Felt::from(l1_gas.amount_with_buffer)
                * Felt::from(l1_gas_price.amount_with_buffer)
                + Felt::from(l2_gas.amount_with_buffer)
                    * Felt::from(l2_gas_price.amount_with_buffer)
                + Felt::from(l1_data_gas.amount_with_buffer)
                    * Felt::from(l1_data_gas_price.amount_with_buffer),
        },
        account_deployment
            .l1_gas(l1_gas.amount_with_buffer)
            .l1_gas_price(l1_gas_price.amount_with_buffer)
            .l2_gas(l2_gas.amount_with_buffer)
            .l2_gas_price(l2_gas_price.amount_with_buffer)
            .l1_data_gas(l1_data_gas.amount_with_buffer)
            .l1_data_gas_price(l1_data_gas_price.amount_with_buffer),
    ))
}

fn resolve_amount_buffer<T>(manual: &Option<T>, estimate: Felt) -> Result<AmountWithBuffer<T>>
where
    T: Copy + TryFrom<Felt>,
{
    Ok(match manual {
        // No buffer applied when a manual override exists
        Some(manual) => AmountWithBuffer {
            amount: *manual,
            amount_with_buffer: *manual,
        },
        // TODO: make buffer configurable
        None => AmountWithBuffer {
            amount: estimate
                .try_into()
                .map_err(|_| anyhow::anyhow!("gas amount or price overflow"))?,
            amount_with_buffer: ((estimate * Felt::THREE).floor_div(&NonZeroFelt::TWO))
                .try_into()
                .map_err(|_| anyhow::anyhow!("gas amount or price overflow"))?,
        },
    })
}
