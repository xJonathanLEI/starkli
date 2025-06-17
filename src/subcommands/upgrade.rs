use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use starknet::core::types::Felt;
use std::path::PathBuf;

use crate::{
    account::AccountArgs,
    casm::CasmArgs,
    fee::FeeArgs,
    subcommands::{Declare, Invoke},
    verbosity::VerbosityArgs,
    ProviderArgs,
};

#[derive(Debug, Parser)]
pub struct Upgrade {
    // SHARED ARGS
    #[clap(flatten)]
    provider: ProviderArgs,
    #[clap(flatten)]
    account: AccountArgs,
    #[clap(flatten)]
    fee: FeeArgs,
    #[clap(flatten)]
    verbosity: VerbosityArgs,
    #[clap(long, short, help = "Wait for each transaction to confirm before proceeding")]
    watch: bool,
    #[clap(
        long,
        env = "STARKNET_POLL_INTERVAL",
        default_value = "5000",
        help = "Transaction result poll interval in milliseconds"
    )]
    poll_interval: u64,

    // DECLARE-SPECIFIC ARGS
    #[clap(help = "Path to the contract artifact file (json)")]
    file: PathBuf,
    #[clap(flatten)]
    casm: CasmArgs,
    #[clap(long, help = "Do not publish the ABI of the class")]
    no_abi: bool,
    #[clap(long, help = "Provide transaction nonce manually for the declare transaction")]
    nonce: Option<Felt>,

    // UPGRADE-SPECIFIC ARGS
    #[clap(help = "Address of the upgradeable contract to call upgrade on")]
    upgrade_contract: String,
    #[clap(long, default_value = "upgrade", help = "Selector for the upgrade entrypoint")]
    upgrade_selector: String,
}

impl Upgrade {
    pub async fn run(self) -> Result<()> {
        self.verbosity.setup_logging();

        // 1. Declare the new class
        let declare_cmd = Declare {
            provider: self.provider.clone(),
            account: self.account.clone(),
            casm: self.casm,
            fee: self.fee.clone(),
            no_abi: self.no_abi,
            simulate: false, // Not supported for upgrade
            nonce: self.nonce,
            watch: true, // We handle watch logic manually
            poll_interval: self.poll_interval,
            file: self.file,
            verbosity: self.verbosity.clone(),
        };

        eprintln!("Declaring new contract class...");
        let declare_result = declare_cmd.run_as_lib().await?;
        eprintln!("Declaration transaction: {:#064x}", declare_result.transaction_hash);

        let class_hash = declare_result.class_hash;
        eprintln!("Successfully declared class: {:#064x}", class_hash);

        // 2. Invoke the upgrade function
        eprintln!("\nInvoking upgrade function on contract: {}", self.upgrade_contract.as_str());

        let calls = vec![
            self.upgrade_contract,
            self.upgrade_selector,
            format!("{:#064x}", class_hash),
        ];

        let invoke_cmd = Invoke {
            provider: self.provider,
            account: self.account,
            fee: self.fee,
            simulate: false, // Not supported for upgrade
            nonce: self.nonce.map(|n| n + 1),
            watch: self.watch,
            poll_interval: self.poll_interval,
            calls,
            verbosity: self.verbosity,
        };

        invoke_cmd.run().await?;

        eprintln!("\nUpgrade complete. New class hash:");
        println!("{}", format!("{:#064x}", class_hash).bright_yellow());

        Ok(())
    }
}
