use std::{sync::Arc, time::Duration};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use colored_json::{ColorMode, Output};
use starknet::{
    accounts::{Account, Call, ConnectedAccount},
    core::types::FieldElement,
    macros::felt,
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

#[derive(Debug, Parser)]
pub struct Invoke {
    #[clap(flatten)]
    provider: ProviderArgs,
    #[clap(flatten)]
    account: AccountArgs,
    #[clap(flatten)]
    fee: FeeArgs,
    #[clap(long, help = "Simulate the transaction only")]
    simulate: bool,
    #[clap(long, help = "Print the raw transaction data")]
    raw_tx: bool,
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
    #[clap(help = "One or more contract calls. See documentation for more details")]
    calls: Vec<String>,
    #[clap(flatten)]
    verbosity: VerbosityArgs,
}

impl Invoke {
    pub async fn run(self) -> Result<()> {
        self.verbosity.setup_logging();

        let fee_setting = self.fee.into_setting()?;
        if self.simulate && fee_setting.is_estimate_only() {
            anyhow::bail!("--simulate cannot be used with --estimate-only");
        } else if self.simulate && self.raw_tx {
            anyhow::bail!("--simulate cannot be used with --raw-tx");
        } else if self.raw_tx && fee_setting.is_estimate_only() {
            anyhow::bail!("--estimate-only cannot be used with --raw-tx");
        }

        let provider = Arc::new(self.provider.into_provider()?);
        let felt_decoder = FeltDecoder::new(AddressBookResolver::new(provider.clone()));

        // Parses and resolves the calls
        let calls = {
            // TODO: show more helpful message
            let unexpected_end_of_args = || anyhow::anyhow!("unexpected end of arguments");

            let mut buffer = vec![];

            let mut arg_iter = self.calls.into_iter();

            while let Some(first_arg) = arg_iter.next() {
                let contract_address = felt_decoder
                    .decode_single_with_addr_fallback(&first_arg)
                    .await?;

                let next_arg = arg_iter.next().ok_or_else(unexpected_end_of_args)?;
                let selector = felt_decoder
                    .decode_single_with_selector_fallback(&next_arg)
                    .await?;

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

        let account = self.account.into_account(provider.clone()).await?;

        let execution = account.execute(calls).fee_estimate_multiplier(1.5f64);

        let max_fee = match fee_setting {
            FeeSetting::Manual(fee) => fee,
            FeeSetting::EstimateOnly | FeeSetting::None => {
                let estimated_fee = execution.estimate_fee().await?.overall_fee;

                if fee_setting.is_estimate_only() {
                    println!(
                        "{} ETH",
                        format!("{}", estimated_fee.to_big_decimal(18)).bright_yellow(),
                    );
                    return Ok(());
                }

                // TODO: make buffer configurable
                (estimated_fee * felt!("3")).floor_div(felt!("2"))
            }
        };

        let execution = match self.nonce {
            Some(nonce) => execution.nonce(nonce),
            None => execution,
        };
        let execution = execution.max_fee(max_fee);

        if self.raw_tx {
            let nonce = match self.nonce {
                Some(nonce) => nonce,
                None => account.get_nonce().await?,
            };
            let execution = execution.nonce(nonce);
            let raw_tx = execution.prepared()?.get_invoke_request(true).await?;
            let raw_tx_json = serde_json::to_value(raw_tx)?;
            let raw_tx_json =
                colored_json::to_colored_json(&raw_tx_json, ColorMode::Auto(Output::StdOut))?;
            println!("{raw_tx_json}");
            return Ok(());
        }

        if self.simulate {
            let simulation = execution.simulate(false, false).await?;
            let simulation_json = serde_json::to_value(simulation)?;

            let simulation_json =
                colored_json::to_colored_json(&simulation_json, ColorMode::Auto(Output::StdOut))?;
            println!("{simulation_json}");
            return Ok(());
        }

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
            watch_tx(
                &provider,
                invoke_tx,
                Duration::from_millis(self.poll_interval),
            )
            .await?;
        }

        Ok(())
    }
}
