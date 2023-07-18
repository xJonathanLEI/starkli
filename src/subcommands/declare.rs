use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use starknet::{
    accounts::{Account, SingleOwnerAccount},
    core::types::{
        contract::{legacy::LegacyContractClass, CompiledClass, SierraClass},
        BlockId, BlockTag, FieldElement, StarknetError,
    },
    providers::{MaybeUnknownErrorCode, Provider, ProviderError, StarknetErrorWithMessage},
};

use crate::{
    account::{AccountConfig, DeploymentStatus},
    casm::{CasmArgs, CasmHashSource},
    fee::{FeeArgs, FeeSetting},
    path::ExpandedPathbufParser,
    signer::SignerArgs,
    utils::watch_tx,
    verbosity::VerbosityArgs,
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
    #[clap(
        long,
        env = "STARKNET_ACCOUNT",
        value_parser = ExpandedPathbufParser,
        help = "Path to account config JSON file"
    )]
    account: PathBuf,
    #[clap(flatten)]
    fee: FeeArgs,
    #[clap(long, help = "Wait for the transaction to confirm")]
    watch: bool,
    #[clap(
        value_parser = ExpandedPathbufParser,
        help = "Path to contract artifact file"
    )]
    file: PathBuf,
    #[clap(flatten)]
    verbosity: VerbosityArgs,
}

impl Declare {
    pub async fn run(self) -> Result<()> {
        self.verbosity.setup_logging();

        let fee_setting = self.fee.into_setting()?;

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

        // Workaround for issue:
        //   https://github.com/eqlabs/pathfinder/issues/1208
        let (fee_multiplier_num, fee_multiplier_denom) =
            if provider.is_rpc() { (5, 2) } else { (3, 2) };

        // Working around a deserialization bug in `starknet-rs`:
        //   https://github.com/xJonathanLEI/starknet-rs/issues/392

        #[allow(clippy::redundant_pattern_matching)]
        let (class_hash, declaration_tx_hash) = if let Ok(class) =
            serde_json::from_reader::<_, SierraClass>(std::fs::File::open(&self.file)?)
        {
            // Declaring Cairo 1 class
            let class_hash = class.class_hash()?;

            // TODO: add option to skip checking
            if Self::check_already_declared(&provider, class_hash).await? {
                return Ok(());
            }

            let casm_source = self.casm.into_casm_hash_source(&provider).await?;

            if !fee_setting.is_estimate_only() {
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

            if !fee_setting.is_estimate_only() {
                eprintln!(
                    "CASM class hash: {}",
                    format!("{:#064x}", casm_class_hash).bright_yellow()
                );
            }

            // TODO: make buffer configurable
            let declaration = account.declare(Arc::new(class.flatten()?), casm_class_hash);

            let max_fee = match fee_setting {
                FeeSetting::Manual(fee) => fee,
                FeeSetting::EstimateOnly | FeeSetting::None => {
                    let estimated_fee = declaration.estimate_fee().await?.overall_fee;

                    if fee_setting.is_estimate_only() {
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
                    let estimated_fee_with_buffer =
                        estimated_fee * fee_multiplier_num / fee_multiplier_denom;

                    estimated_fee_with_buffer.into()
                }
            };

            (
                class_hash,
                declaration.max_fee(max_fee).send().await?.transaction_hash,
            )
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

            // TODO: add option to skip checking
            if Self::check_already_declared(&provider, class_hash).await? {
                return Ok(());
            }

            if !fee_setting.is_estimate_only() {
                eprintln!(
                    "Declaring Cairo 0 (deprecated) class: {}",
                    format!("{:#064x}", class_hash).bright_yellow()
                );
            }

            // TODO: make buffer configurable
            let declaration = account.declare_legacy(Arc::new(class));

            let max_fee = match fee_setting {
                FeeSetting::Manual(fee) => fee,
                FeeSetting::EstimateOnly | FeeSetting::None => {
                    let estimated_fee = declaration.estimate_fee().await?.overall_fee;

                    if fee_setting.is_estimate_only() {
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
                    let estimated_fee_with_buffer =
                        estimated_fee * fee_multiplier_num / fee_multiplier_denom;

                    estimated_fee_with_buffer.into()
                }
            };

            (
                class_hash,
                declaration.max_fee(max_fee).send().await?.transaction_hash,
            )
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

    async fn check_already_declared<P>(provider: P, class_hash: FieldElement) -> Result<bool>
    where
        P: Provider,
        P::Error: 'static,
    {
        match provider
            .get_class(BlockId::Tag(BlockTag::Pending), class_hash)
            .await
        {
            Ok(_) => {
                eprintln!("Not declaring class as it's already declared. Class hash:");
                println!("{}", format!("{:#064x}", class_hash).bright_yellow());

                Ok(true)
            }
            Err(ProviderError::StarknetError(StarknetErrorWithMessage {
                code: MaybeUnknownErrorCode::Known(StarknetError::ClassHashNotFound),
                ..
            })) => Ok(false),
            Err(err) => Err(err.into()),
        }
    }
}
