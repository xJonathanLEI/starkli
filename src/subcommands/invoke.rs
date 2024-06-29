use std::{sync::Arc, time::Duration};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use starknet::{
    accounts::{Account, Call},
    core::types::Felt,
};

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

        let invoke_tx = match fee_setting {
            FeeSetting::Eth(fee_setting) => {
                let execution = account.execute_v1(calls).fee_estimate_multiplier(1.5f64);
                let execution = match self.nonce {
                    Some(nonce) => execution.nonce(nonce),
                    None => execution,
                };

                let execution = match fee_setting {
                    TokenFeeSetting::EstimateOnly => {
                        let estimated_fee = execution
                            .estimate_fee()
                            .await
                            .map_err(account_error_mapper)?
                            .overall_fee;

                        println!(
                            "{} ETH",
                            format!("{}", felt_to_bigdecimal(estimated_fee, 18)).bright_yellow(),
                        );
                        return Ok(());
                    }
                    TokenFeeSetting::Manual(fee) => execution.max_fee(fee.max_fee),
                    TokenFeeSetting::None => execution,
                };

                if self.simulate {
                    print_colored_json(&execution.simulate(false, false).await?)?;
                    return Ok(());
                }

                execution.send().await
            }
            FeeSetting::Strk(fee_setting) => {
                let execution = account.execute_v3(calls);
                let execution = match self.nonce {
                    Some(nonce) => execution.nonce(nonce),
                    None => execution,
                };

                let execution = match fee_setting {
                    TokenFeeSetting::EstimateOnly => {
                        let estimated_fee = execution
                            .estimate_fee()
                            .await
                            .map_err(account_error_mapper)?;

                        println!(
                            "{} STRK",
                            format!("{}", felt_to_bigdecimal(estimated_fee.overall_fee, 18))
                                .bright_yellow(),
                        );
                        return Ok(());
                    }
                    TokenFeeSetting::Manual(fee) => {
                        let execution = if let Some(gas) = fee.gas {
                            execution.gas(gas)
                        } else {
                            execution
                        };

                        if let Some(gas_price) = fee.gas_price {
                            execution.gas_price(gas_price)
                        } else {
                            execution
                        }
                    }
                    TokenFeeSetting::None => execution,
                };

                if self.simulate {
                    print_colored_json(&execution.simulate(false, false).await?)?;
                    return Ok(());
                }

                execution.send().await
            }
        }
        .map_err(account_error_mapper)?
        .transaction_hash;

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
