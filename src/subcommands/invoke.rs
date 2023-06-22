use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use starknet::{
    accounts::{Account, Call, SingleOwnerAccount},
    core::{types::FieldElement, utils::get_selector_from_name},
    providers::Provider,
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
pub struct Invoke {
    #[clap(flatten)]
    provider: ProviderArgs,
    #[clap(flatten)]
    signer: SignerArgs,
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
    #[clap(help = "One or more contract calls. See documentation for more details")]
    calls: Vec<String>,
}

impl Invoke {
    pub async fn run(self) -> Result<()> {
        let provider = Arc::new(self.provider.into_provider());
        let felt_decoder = FeltDecoder::new(AddressBookResolver::new(provider.clone()));

        if !self.account.exists() {
            anyhow::bail!("account config file not found");
        }

        // Parses and resolves the calls
        let calls = {
            // TODO: show more helpful message
            let unexpected_end_of_args = || anyhow::anyhow!("unexpected end of arguments");

            let mut buffer = vec![];

            let mut arg_iter = self.calls.into_iter();

            while let Some(first_arg) = arg_iter.next() {
                let contract_address = felt_decoder.decode_single(&first_arg).await?;

                let next_arg = arg_iter.next().ok_or_else(unexpected_end_of_args)?;
                let selector = get_selector_from_name(&next_arg)?;

                let mut calldata = vec![];
                for arg in &mut arg_iter {
                    let mut arg = match arg.as_str() {
                        "/" | "-" | "\\" => break,
                        _ => felt_decoder.decode(&arg).await?,
                    };
                    calldata.append(&mut arg);
                }

                buffer.push(Call {
                    to: contract_address,
                    selector,
                    calldata,
                });
            }

            buffer
        };

        if calls.is_empty() {
            anyhow::bail!("empty execution");
        }

        // TODO: refactor account & signer loading

        let account_config: AccountConfig =
            serde_json::from_reader(&mut std::fs::File::open(&self.account)?)?;

        let account_address = match account_config.deployment {
            DeploymentStatus::Undeployed(_) => anyhow::bail!("account not deployed"),
            DeploymentStatus::Deployed(inner) => inner.address,
        };

        let chain_id = provider.chain_id().await?;

        let signer = Arc::new(self.signer.into_signer()?);
        let account = SingleOwnerAccount::new(provider.clone(), signer, account_address, chain_id);

        let execution = account.execute(calls).fee_estimate_multiplier(1.5f64);

        if self.estimate_only {
            let estimated_fee = execution.estimate_fee().await?.overall_fee;

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
        let invoke_tx = execution.send().await?.transaction_hash;
        eprintln!(
            "Invoke transaction: {}",
            format!("{:#064x}", invoke_tx).bright_yellow()
        );

        if self.watch {
            eprintln!(
                "Waiting for transaction {} to confirm...",
                format!("{:#064x}", invoke_tx).bright_yellow(),
            );
            watch_tx(&provider, invoke_tx).await?;
        }

        Ok(())
    }
}
