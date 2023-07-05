use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use starknet::{
    accounts::{Account, SingleOwnerAccount},
    core::types::{
        contract::{legacy::LegacyContractClass, CompiledClass, SierraClass},
        BlockId, BlockTag, FieldElement,
    },
    providers::Provider,
};

use crate::{
    account::{AccountConfig, DeploymentStatus},
    casm::{CasmArgs, CasmHashSource},
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
    #[clap(flatten)]
    casm: CasmArgs,
    #[clap(long, help = "Maximum fee to pay for the transaction")]
    max_fee: Option<FieldElement>,
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
    #[clap(help = "Path to contract artifact file")]
    file: PathBuf,
}

impl Declare {
    pub async fn run(self) -> Result<()> {
        let provider = Arc::new(self.provider.into_provider());

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

        let signer = self.signer.into_signer()?;
        let mut account =
            SingleOwnerAccount::new(provider.clone(), signer, account_address, chain_id);
        account.set_block_id(BlockId::Tag(BlockTag::Pending));

        // TODO: check if class has already been declared

        // Working around a deserialization bug in `starknet-rs`:
        //   https://github.com/xJonathanLEI/starknet-rs/issues/392

        #[allow(clippy::redundant_pattern_matching)]
        let (class_hash, declaration_tx_hash) = if let Ok(class) =
            serde_json::from_reader::<_, SierraClass>(std::fs::File::open(&self.file)?)
        {
            // Declaring Cairo 1 class
            let class_hash = class.class_hash()?;

            let casm_source = self.casm.into_casm_hash_source(&provider).await?;

            if !self.estimate_only {
                eprintln!(
                    "Declaring Cairo 1 class: {}",
                    format!("{:#064x}", class_hash).bright_yellow()
                );

                match &casm_source {
                    CasmHashSource::BuiltInCompiler(compiler) => {
                        eprintln!(
                            "Compiling Sierra class to CASM with compiler version {}...",
                            format!("{}", compiler.version()).bright_yellow()
                        );
                    }
                    CasmHashSource::Hash(hash) => {
                        eprintln!(
                            "Using the provided CASM hash: {}...",
                            format!("{:#064x}", hash).bright_yellow()
                        );
                    }
                }
            }

            let casm_class_hash = casm_source.get_casm_hash(&class)?;

            if !self.estimate_only {
                eprintln!(
                    "CASM class hash: {}",
                    format!("{:#064x}", casm_class_hash).bright_yellow()
                );
            }

            // TODO: make buffer configurable
            let declaration = if let Some(max_fee) = self.max_fee {
                account
                    .declare(Arc::new(class.flatten()?), casm_class_hash)
                    .max_fee(max_fee)
            } else {
                account
                    .declare(Arc::new(class.flatten()?), casm_class_hash)
                    .fee_estimate_multiplier(1.5f64)
            };

            if self.estimate_only {
                let estimated_fee = declaration.estimate_fee().await?.overall_fee;

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

            if !self.estimate_only {
                eprintln!(
                    "Declaring Cairo 0 (deprecated) class: {}",
                    format!("{:#064x}", class_hash).bright_yellow()
                );
            }

            // TODO: make buffer configurable
            let declaration = if let Some(max_fee) = self.max_fee {
                account.declare_legacy(Arc::new(class)).max_fee(max_fee)
            } else {
                account
                    .declare_legacy(Arc::new(class))
                    .fee_estimate_multiplier(1.5f64)
            };

            if self.estimate_only {
                let estimated_fee = declaration.estimate_fee().await?.overall_fee;

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
