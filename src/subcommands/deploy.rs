use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use starknet::{
    accounts::SingleOwnerAccount, contract::ContractFactory, core::types::FieldElement,
    providers::Provider, signers::SigningKey,
};

use crate::{
    account::{AccountConfig, DeploymentStatus},
    address_book::AddressBookResolver,
    decode::FeltDecoder,
    signer::SignerArgs,
    utils::watch_tx,
    ProviderArgs,
};

#[derive(Debug, Parser)]
pub struct Deploy {
    #[clap(flatten)]
    provider: ProviderArgs,
    #[clap(flatten)]
    signer: SignerArgs,
    #[clap(long, help = "Do not derive contract address from deployer address")]
    not_unique: bool,
    #[clap(
        long,
        env = "STARKNET_ACCOUNT",
        help = "Path to account config JSON file"
    )]
    account: PathBuf,
    #[clap(
        long,
        help = "Only estimate transaction fee without sending transaction"
    )]
    estimate_only: bool,
    #[clap(long, help = "Wait for the transaction to confirm")]
    watch: bool,
    #[clap(help = "Class hash")]
    class_hash: String,
    #[clap(help = "Raw constructor arguments")]
    ctor_args: Vec<String>,
}

impl Deploy {
    pub async fn run(self) -> Result<()> {
        let provider = Arc::new(self.provider.into_provider());
        let felt_decoder = FeltDecoder::new(AddressBookResolver::new(provider.clone()));

        if !self.account.exists() {
            anyhow::bail!("account config file not found");
        }

        let class_hash = FieldElement::from_hex_be(&self.class_hash)?;
        let mut ctor_args = vec![];
        for element in self.ctor_args.iter() {
            ctor_args.append(&mut felt_decoder.decode(element).await?);
        }

        // TODO: add option for manually setting salt
        let salt = SigningKey::from_random().secret_scalar();

        // TODO: refactor account & signer loading

        let account_config: AccountConfig =
            serde_json::from_reader(&mut std::fs::File::open(&self.account)?)?;

        let account_address = match account_config.deployment {
            DeploymentStatus::Undeployed(_) => anyhow::bail!("account not deployed"),
            DeploymentStatus::Deployed(inner) => inner.address,
        };

        let chain_id = provider.chain_id().await?;

        let signer = Arc::new(self.signer.into_signer()?);
        let account =
            SingleOwnerAccount::new(provider.clone(), signer.clone(), account_address, chain_id);

        // TODO: allow custom UDC
        let factory = ContractFactory::new(class_hash, account);

        // TODO: pre-compute and show target deployment address

        let contract_deployment = factory.deploy(&ctor_args, salt, !self.not_unique);

        // TODO: add option for manually specifying fees
        let estimated_fee = contract_deployment.estimate_fee().await?.overall_fee;
        if self.estimate_only {
            println!(
                "{} ETH",
                format!(
                    "{}",
                    <u64 as Into<FieldElement>>::into(estimated_fee).to_big_decimal(18)
                )
                .bright_yellow(),
            );
            return Ok(());
        }

        // TODO: make buffer configurable
        let estimated_fee_with_buffer = estimated_fee * 3 / 2;

        eprintln!(
            "Deploying class {} with salt {}...",
            format!("{:#064x}", class_hash).bright_yellow(),
            format!("{:#064x}", salt).bright_yellow()
        );

        let deployment_tx = contract_deployment
            .max_fee(estimated_fee_with_buffer.into())
            .send()
            .await?
            .transaction_hash;
        eprintln!(
            "Contract deployment transaction: {}",
            format!("{:#064x}", deployment_tx).bright_yellow()
        );

        if self.watch {
            eprintln!(
                "Waiting for transaction {} to confirm...",
                format!("{:#064x}", deployment_tx).bright_yellow(),
            );
            watch_tx(&provider, deployment_tx).await?;
        }

        Ok(())
    }
}
