use std::{sync::Arc, time::Duration};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use colored_json::{ColorMode, Output};
use starknet::{
    contract::ContractFactory, core::types::FieldElement, macros::felt, signers::SigningKey,
};

use crate::{
    account::AccountArgs,
    address_book::AddressBookResolver,
    decode::FeltDecoder,
    fee::{FeeArgs, FeeSetting},
    utils::watch_tx,
    verbosity::VerbosityArgs,
    ProviderArgs,
};

/// The default UDC address: 0x041a78e741e5af2fec34b695679bc6891742439f7afb8484ecd7766661ad02bf.
const DEFAULT_UDC_ADDRESS: FieldElement = FieldElement::from_mont([
    15144800532519055890,
    15685625669053253235,
    9333317513348225193,
    121672436446604875,
]);

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
    nonce: Option<FieldElement>,
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

        let class_hash = FieldElement::from_hex_be(&self.class_hash)?;
        let mut ctor_args = vec![];
        for element in self.ctor_args.iter() {
            ctor_args.append(&mut felt_decoder.decode(element).await?);
        }

        let salt = if let Some(s) = self.salt {
            FieldElement::from_hex_be(&s)?
        } else {
            SigningKey::from_random().secret_scalar()
        };

        let account = self.account.into_account(provider.clone()).await?;

        // TODO: allow custom UDC
        let factory = ContractFactory::new_with_udc(class_hash, account, DEFAULT_UDC_ADDRESS);

        let contract_deployment = factory.deploy(ctor_args, salt, !self.not_unique);
        let deployed_address = contract_deployment.deployed_address();

        let max_fee = match fee_setting {
            FeeSetting::Manual(fee) => fee,
            FeeSetting::EstimateOnly | FeeSetting::None => {
                let estimated_fee = contract_deployment.estimate_fee().await?.overall_fee;

                if fee_setting.is_estimate_only() {
                    eprintln!(
                        "{} ETH",
                        format!("{}", estimated_fee.to_big_decimal(18)).bright_yellow(),
                    );
                    return Ok(());
                }

                // TODO: make buffer configurable
                (estimated_fee * felt!("3")).floor_div(felt!("2"))
            }
        };

        eprintln!(
            "Deploying class {} with salt {}...",
            format!("{:#064x}", class_hash).bright_yellow(),
            format!("{:#064x}", salt).bright_yellow()
        );
        eprintln!(
            "The contract will be deployed at address {}",
            format!("{:#064x}", deployed_address).bright_yellow()
        );

        let contract_deployment = match self.nonce {
            Some(nonce) => contract_deployment.nonce(nonce),
            None => contract_deployment,
        };
        let contract_deployment = contract_deployment.max_fee(max_fee);

        if self.simulate {
            let simulation = contract_deployment.simulate(false, false).await?;
            let simulation_json = serde_json::to_value(simulation)?;

            let simulation_json =
                colored_json::to_colored_json(&simulation_json, ColorMode::Auto(Output::StdOut))?;
            println!("{simulation_json}");
            return Ok(());
        }

        let deployment_tx = contract_deployment.send().await?.transaction_hash;
        eprintln!(
            "Contract deployment transaction: {}",
            format!("{:#064x}", deployment_tx).bright_yellow()
        );

        if self.watch {
            eprintln!(
                "Waiting for transaction {} to confirm...",
                format!("{:#064x}", deployment_tx).bright_yellow(),
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
        println!("{}", format!("{:#064x}", deployed_address).bright_yellow());

        Ok(())
    }
}
