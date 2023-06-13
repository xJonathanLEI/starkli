use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use starknet::{
    accounts::{Account, SingleOwnerAccount},
    core::types::contract::{legacy::LegacyContractClass, CompiledClass, SierraClass},
    providers::Provider,
};

use crate::{
    account::{AccountConfig, DeploymentStatus},
    signer::SignerArgs,
    utils::watch_tx,
    ProviderArgs,
};

#[derive(Debug, Parser)]
pub struct Declare {
    #[clap(flatten)]
    provider: ProviderArgs,
    #[clap(flatten)]
    signer: SignerArgs,
    #[clap(long, help = "Path to account config JSON file")]
    account: PathBuf,
    #[clap(long, help = "Wait for the transaction to confirm")]
    watch: bool,
    #[clap(help = "Path to contract artifact file")]
    file: PathBuf,
}

impl Declare {
    pub async fn run(self) -> Result<()> {
        let provider = Arc::new(self.provider.into_provider());
        let signer = self.signer.into_signer()?;

        if !self.account.exists() {
            anyhow::bail!("account config file not found");
        }

        let account_config: AccountConfig =
            serde_json::from_reader(&mut std::fs::File::open(&self.account)?)?;

        let account_address = match account_config.deployment {
            DeploymentStatus::Undeployed(_) => anyhow::bail!("account not deployed"),
            DeploymentStatus::Deployed(inner) => inner.address,
        };

        let chain_id = provider.chain_id().await?;

        let account = SingleOwnerAccount::new(provider.clone(), signer, account_address, chain_id);

        // TODO: check if class has already been declared

        // Working around a deserialization bug in `starknet-rs`:
        //   https://github.com/xJonathanLEI/starknet-rs/issues/392

        #[allow(clippy::redundant_pattern_matching)]
        let (class_hash, declaration_tx_hash) = if let Ok(class) =
            serde_json::from_reader::<_, SierraClass>(std::fs::File::open(&self.file)?)
        {
            // Declaring Cairo 1 class
            let class_hash = class.class_hash()?;

            eprintln!(
                "Declaring Cairo 1 class: {}",
                format!("{:#064x}", class_hash).bright_yellow()
            );
            eprintln!("Compiling Sierra class to CASM with compiler version v1.1.0...");

            // Code adapted from the `starknet-sierra-compile` CLI

            // TODO: directly convert type without going through JSON
            let contract_class: cairo_lang_starknet::contract_class::ContractClass =
                serde_json::from_str(&serde_json::to_string(&class)?)?;

            // TODO: implement the `validate_compatible_sierra_version` call

            // TODO: allow manually specifying casm hash (for advanced users)
            // TODO: add `starknet-sierra-compile` version to CLI version display
            // TODO: allow shelling out local `starknet-sierra-compile` installations
            let casm_contract =
                cairo_lang_starknet::casm_contract_class::CasmContractClass::from_contract_class(
                    contract_class,
                    false,
                )?;

            // TODO: directly convert type without going through JSON
            let casm_class =
                serde_json::from_str::<CompiledClass>(&serde_json::to_string(&casm_contract)?)?;

            let casm_class_hash = casm_class.class_hash()?;

            eprintln!(
                "CASM class hash: {}",
                format!("{:#064x}", casm_class_hash).bright_yellow()
            );

            // TODO: make buffer configurable
            let declaration = account
                .declare(Arc::new(class.flatten()?), casm_class_hash)
                .fee_estimate_multiplier(1.5f64);

            (class_hash, declaration.send().await?.transaction_hash)
        } else if let Ok(_) =
            serde_json::from_reader::<_, CompiledClass>(std::fs::File::open(&self.file)?)
        {
            // TODO: add more helpful instructions to fix this
            anyhow::bail!("unexpected CASM class");
        } else if let Ok(class) =
            serde_json::from_reader::<_, LegacyContractClass>(std::fs::File::open(self.file)?)
        {
            // Declaring Cairo 0 class
            let class_hash = class.class_hash()?;

            eprintln!(
                "Declaring Cairo 0 (deprecated) class: {}",
                format!("{:#064x}", class_hash).bright_yellow()
            );

            // TODO: make buffer configurable
            let declaration = account
                .declare_legacy(Arc::new(class))
                .fee_estimate_multiplier(1.5f64);

            (class_hash, declaration.send().await?.transaction_hash)
        } else {
            anyhow::bail!("failed to parse contract artifact");
        };

        eprintln!(
            "Contract declaration transaction: {}",
            format!("{:#064x}", declaration_tx_hash).bright_yellow()
        );

        if self.watch {
            eprintln!(
                "Waiting for transaction {} to confirm...",
                format!("{:#064x}", declaration_tx_hash).bright_yellow(),
            );
            watch_tx(&provider, declaration_tx_hash).await?;
        }

        eprintln!("Class hash declared:");

        // Only the class hash goes to stdout so this can be easily scripted
        println!("{}", format!("{:#064x}", class_hash).bright_yellow());

        Ok(())
    }
}
