
use anyhow::Result;
use clap::Parser;
use starknet::core::types::Felt;

use crate::{
    account::AccountArgs,
    fee::{FeeArgs},
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
    #[clap(
        long,
        env = "STARKNET_POLL_INTERVAL",
        default_value = "5000",
        help = "Transaction result poll interval in milliseconds"
    )]
    pub poll_interval: u64,
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
            poll_interval: self.poll_interval,
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
            nonce: self.nonce.map(|n| n + 1),
            watch: self.watch,
            poll_interval: self.poll_interval,
            calls,
            verbosity: self.verbosity,
        };
        invoke_cmd.run().await?;
        Ok(())
    }
}
