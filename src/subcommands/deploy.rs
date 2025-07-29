use std::{sync::Arc, time::Duration};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use starknet::{contract::ContractFactory, core::types::Felt, macros::felt, signers::SigningKey};

use crate::{
    account::AccountArgs,
    address_book::AddressBookResolver,
    decode::FeltDecoder,
    error::account_error_mapper,
    fee::{FeeArgs, FeeSetting, TokenFeeSetting},
    utils::{felt_to_bigdecimal, print_colored_json, watch_tx},
    verbosity::VerbosityArgs,
    ProviderArgs,
};

/// The default UDC address: 0x041a78e741e5af2fec34b695679bc6891742439f7afb8484ecd7766661ad02bf.
const DEFAULT_UDC_ADDRESS: Felt =
    felt!("0x041a78e741e5af2fec34b695679bc6891742439f7afb8484ecd7766661ad02bf");

#[derive(Debug, Parser)]
pub struct Deploy {
    #[clap(flatten)]
    provider: ProviderArgs,
    #[clap(flatten)]
    account: AccountArgs,
    #[clap(long, help = "Do not derive contract address from deployer address")]
    not_unique: bool,
    #[clap(flatten)]
    fee: FeeArgs,
    #[clap(long, help = "Simulate the transaction only")]
    simulate: bool,
    #[clap(long, help = "Use the given salt to compute contract deploy address")]
    salt: Option<String>,
    #[clap(long, help = "Provide transaction nonce manually")]
    nonce: Option<Felt>,
    #[clap(long, short, help = "Wait for the transaction to confirm")]
    watch: bool,
    #[clap(
        long,
        env = "STARKNET_POLL_INTERVAL",
        default_value = "5000",
        help = "Transaction result poll interval in milliseconds"
    )]
    poll_interval: u64,
    #[clap(help = "Class hash")]
    class_hash: String,
    #[clap(help = "Raw constructor arguments")]
    ctor_args: Vec<String>,
    #[clap(flatten)]
    verbosity: VerbosityArgs,
}

impl Deploy {
    pub async fn run(self) -> Result<()> {
        self.verbosity.setup_logging();

        let fee_setting = self.fee.into_setting()?;
        if self.simulate && fee_setting.is_estimate_only() {
            anyhow::bail!("--simulate cannot be used with --estimate-only");
        }

        let provider = Arc::new(self.provider.into_provider()?);
        let felt_decoder = FeltDecoder::new(AddressBookResolver::new(provider.clone()));

        let class_hash = Felt::from_hex(&self.class_hash)?;
        let mut ctor_args = vec![];
        for element in self.ctor_args.iter() {
            ctor_args.append(&mut felt_decoder.decode(element).await?);
        }

        let salt = if let Some(s) = self.salt {
            Felt::from_hex(&s)?
        } else {
            SigningKey::from_random().secret_scalar()
        };

        let account = self.account.into_account(provider.clone()).await?;

        // TODO: allow custom UDC
        let factory = ContractFactory::new_with_udc(class_hash, account, DEFAULT_UDC_ADDRESS);

        let deployed_address = factory
            .deploy_v3(ctor_args.clone(), salt, !self.not_unique)
            .deployed_address();

        if !fee_setting.is_estimate_only() {
            eprintln!(
                "Deploying class {} with salt {}...",
                format!("{class_hash:#064x}").bright_yellow(),
                format!("{salt:#064x}").bright_yellow()
            );
            eprintln!(
                "The contract will be deployed at address {}",
                format!("{deployed_address:#064x}").bright_yellow()
            );
        }

        let deployment_tx = match fee_setting {
            FeeSetting::Strk(fee_setting) => {
                let contract_deployment = factory.deploy_v3(ctor_args, salt, !self.not_unique);
                let contract_deployment = match self.nonce {
                    Some(nonce) => contract_deployment.nonce(nonce),
                    None => contract_deployment,
                };

                let contract_deployment = match fee_setting {
                    TokenFeeSetting::EstimateOnly => {
                        let estimated_fee = contract_deployment
                            .estimate_fee()
                            .await
                            .map_err(account_error_mapper)?
                            .overall_fee;

                        eprintln!(
                            "{} STRK",
                            format!("{}", felt_to_bigdecimal(estimated_fee, 18)).bright_yellow(),
                        );
                        return Ok(());
                    }
                    TokenFeeSetting::Manual(fee) => {
                        let contract_deployment = if let Some(l1_gas) = fee.l1_gas {
                            contract_deployment.l1_gas(l1_gas)
                        } else {
                            contract_deployment
                        };
                        let contract_deployment = if let Some(l2_gas) = fee.l2_gas {
                            contract_deployment.l2_gas(l2_gas)
                        } else {
                            contract_deployment
                        };
                        let contract_deployment = if let Some(l1_data_gas) = fee.l1_data_gas {
                            contract_deployment.l1_data_gas(l1_data_gas)
                        } else {
                            contract_deployment
                        };

                        let contract_deployment = if let Some(l1_gas_price) = fee.l1_gas_price {
                            contract_deployment.l1_gas_price(l1_gas_price)
                        } else {
                            contract_deployment
                        };
                        let contract_deployment = if let Some(l2_gas_price) = fee.l2_gas_price {
                            contract_deployment.l2_gas_price(l2_gas_price)
                        } else {
                            contract_deployment
                        };
                        if let Some(l1_data_gas_price) = fee.l1_data_gas_price {
                            contract_deployment.l1_data_gas_price(l1_data_gas_price)
                        } else {
                            contract_deployment
                        }
                    }
                    TokenFeeSetting::None => contract_deployment,
                };

                if self.simulate {
                    print_colored_json(&contract_deployment.simulate(false, false).await?)?;
                    return Ok(());
                }

                contract_deployment.send().await
            }
        }
        .map_err(account_error_mapper)?
        .transaction_hash;

        eprintln!(
            "Contract deployment transaction: {}",
            format!("{deployment_tx:#064x}").bright_yellow()
        );

        if self.watch {
            eprintln!(
                "Waiting for transaction {} to confirm...",
                format!("{deployment_tx:#064x}").bright_yellow(),
            );
            watch_tx(
                &provider,
                deployment_tx,
                Duration::from_millis(self.poll_interval),
            )
            .await?;
        }

        eprintln!("Contract deployed:");

        // Only the contract goes to stdout so this can be easily scripted
        println!("{}", format!("{deployed_address:#064x}").bright_yellow());

        Ok(())
    }
}
