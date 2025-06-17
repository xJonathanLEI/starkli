use std::{sync::Arc, time::Duration};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use starknet::{
    contract::ContractFactory,
    core::types::{Call, Felt},
    macros::felt,
    signers::SigningKey,
};

use crate::{
    account::AccountArgs,
    fee::{FeeArgs},
    utils::{felt_to_bigdecimal, print_colored_json, watch_tx},
    verbosity::VerbosityArgs,
    ProviderArgs,
    subcommands::{Deploy, Invoke},
};

#[derive(Debug, Parser)]
pub struct Upgrade {
    #[clap(flatten)]
    provider: ProviderArgs,
    #[clap(flatten)]
    account: AccountArgs,
    #[clap(flatten)]
    fee: FeeArgs,
    #[clap(flatten)]
    verbosity: VerbosityArgs,
    #[clap(long, help = "Do not derive contract address from deployer address")]
    not_unique: bool,
    #[clap(long, help = "Use the given salt to compute contract deploy address")]
    salt: Option<String>,
    #[clap(long, help = "Provide transaction nonce manually for the first transaction")]
    nonce: Option<Felt>,
    #[clap(long, short, help = "Wait for the transactions to confirm")]
    watch: bool,
    #[clap(help = "Class hash for the new implementation")]
    class_hash: String,
    #[clap(help = "Raw constructor arguments for the new implementation")]
    ctor_args: Vec<String>,
    #[clap(help = "Address of the upgradeable contract to call upgrade on")]
    upgrade_contract: String,
    #[clap(long, default_value = "upgrade", help = "Selector for the upgrade entrypoint")]
    selector: String,
    #[clap(help = "Additional arguments for the upgrade call (e.g. new class hash)")]
    upgrade_args: Vec<String>,
}

impl Upgrade {
    pub async fn run(self) -> Result<()> {
        self.verbosity.setup_logging();

        // 1. Deploy new class using Deploy subcommand
        let deploy_cmd = Deploy {
            provider: self.provider.clone(),
            account: self.account.clone(),
            not_unique: self.not_unique,
            fee: self.fee.clone(),
            simulate: false,
            salt: self.salt.clone(),
            nonce: self.nonce,
            watch: self.watch,
            poll_interval: 5000,
            class_hash: self.class_hash.clone(),
            ctor_args: self.ctor_args.clone(),
            verbosity: self.verbosity.clone(),
        };
        deploy_cmd.run().await?;

        // 2. Invoke upgrade entrypoint using Invoke subcommand
        let mut calls = vec![self.upgrade_contract, self.selector];
        if self.upgrade_args.is_empty() {
            calls.push(self.class_hash);
        } else {
            calls.extend(self.upgrade_args);
        }

        let invoke_cmd = Invoke {
            provider: self.provider.clone(),
            account: self.account,
            fee: self.fee,
            simulate: false,
            nonce: None,
            watch: self.watch,
            poll_interval: 5000,
            calls,
            verbosity: self.verbosity,
        };
        invoke_cmd.run().await?;
        Ok(())
    }
}
