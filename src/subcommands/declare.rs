use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use starknet::{
    accounts::{Account, SingleOwnerAccount},
    core::types::contract::{legacy::LegacyContractClass, CompiledClass, SierraClass},
    providers::Provider,
    signers::{LocalWallet, SigningKey},
};

use crate::{
    account::{AccountConfig, DeploymentStatus},
    utils::watch_tx,
    ProviderArgs,
};

#[derive(Debug, Parser)]
pub struct Declare {
    #[clap(flatten)]
    provider: ProviderArgs,
    #[clap(long, help = "Path to keystore JSON file")]
    keystore: PathBuf,
    #[clap(
        long,
        help = "Supply keystore password from command line option instead of prompt"
    )]
    keystore_password: Option<String>,
    #[clap(long, help = "Path to account config JSON file")]
    account: PathBuf,
    #[clap(long, help = "Wait for the transaction to confirm")]
    watch: bool,
    #[clap(help = "Path to contract artifact file")]
    file: PathBuf,
}

impl Declare {
    pub async fn run(self) -> Result<()> {
        if self.keystore_password.is_some() {
            eprintln!(
                "{}",
                "WARNING: setting keystore passwords via --password is generally considered \
                insecure, as they will be stored in your shell history or other log files."
                    .bright_magenta()
            );
        }

        let provider = Arc::new(self.provider.into_provider());

        if !self.keystore.exists() {
            anyhow::bail!("keystore file not found");
        }

        if !self.account.exists() {
            anyhow::bail!("account config file not found");
        }

        let account_config: AccountConfig =
            serde_json::from_reader(&mut std::fs::File::open(&self.account)?)?;

        let account_address = match account_config.deployment {
            DeploymentStatus::Undeployed(_) => anyhow::bail!("account not deployed"),
            DeploymentStatus::Deployed(inner) => inner.address,
        };

        let password = if let Some(password) = self.keystore_password {
            password
        } else {
            rpassword::prompt_password("Enter keystore password: ")?
        };

        let key = SigningKey::from_keystore(self.keystore, &password)?;

        let chain_id = provider.chain_id().await?;

        let account = SingleOwnerAccount::new(
            provider.clone(),
            LocalWallet::from_signing_key(key.clone()),
            account_address,
            chain_id,
        );

        // TODO: check if class has already been declared

        // Working around a deserialization bug in `starknet-rs`:
        //   https://github.com/xJonathanLEI/starknet-rs/issues/392

        #[allow(clippy::redundant_pattern_matching)]
        let (class_hash, declaration_tx_hash) = if let Ok(_) =
            serde_json::from_reader::<_, SierraClass>(std::fs::File::open(&self.file)?)
        {
            // Declaring Cairo 1 class
            todo!("Cairo 1 clas declaration not implemented");
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
