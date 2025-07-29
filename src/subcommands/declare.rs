use std::{path::PathBuf, sync::Arc, time::Duration};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use starknet::{
    accounts::Account,
    core::types::{
        contract::{legacy::LegacyContractClass, CompiledClass, SierraClass, SierraClassDebugInfo},
        BlockId, BlockTag, CompressedLegacyContractClass, Felt, FlattenedSierraClass,
        StarknetError,
    },
    providers::{Provider, ProviderError},
};

use crate::{
    account::AccountArgs,
    casm::{CasmArgs, CasmHashSource},
    compiler::BuiltInCompiler,
    error::account_error_mapper,
    fee::{FeeArgs, FeeSetting, TokenFeeSetting},
    path::ExpandedPathbufParser,
    utils::{felt_to_bigdecimal, print_colored_json, watch_tx},
    verbosity::VerbosityArgs,
    ProviderArgs,
};

#[derive(Debug, Parser)]
pub struct Declare {
    #[clap(flatten)]
    provider: ProviderArgs,
    #[clap(flatten)]
    account: AccountArgs,
    #[clap(flatten)]
    casm: CasmArgs,
    #[clap(flatten)]
    fee: FeeArgs,
    #[clap(long, help = "Do not publish the ABI of the class")]
    no_abi: bool,
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
    #[clap(
        value_parser = ExpandedPathbufParser,
        help = "Path to contract artifact file"
    )]
    file: PathBuf,
    #[clap(flatten)]
    verbosity: VerbosityArgs,
}

enum Declarable {
    CairoOne(FlattenedSierraClass),
}

impl Declare {
    pub async fn run(self) -> Result<()> {
        self.verbosity.setup_logging();

        let fee_setting = self.fee.into_setting()?;
        if self.simulate && fee_setting.is_estimate_only() {
            anyhow::bail!("--simulate cannot be used with --estimate-only");
        }

        let provider = Arc::new(self.provider.into_provider()?);

        let account = self.account.into_account(provider.clone()).await?;

        // Working around a deserialization bug in `starknet-rs`:
        //   https://github.com/xJonathanLEI/starknet-rs/issues/392

        let declarable = if let Ok(class) =
            serde_json::from_reader::<_, SierraClass>(std::fs::File::open(&self.file)?)
        {
            Declarable::CairoOne(class.flatten()?)
        } else if let Ok(class) =
            serde_json::from_reader::<_, FlattenedSierraClass>(std::fs::File::open(&self.file)?)
        {
            Declarable::CairoOne(class)
        } else if serde_json::from_reader::<_, LegacyContractClass>(std::fs::File::open(
            &self.file,
        )?)
        .is_ok()
            || serde_json::from_reader::<_, CompressedLegacyContractClass>(std::fs::File::open(
                &self.file,
            )?)
            .is_ok()
        {
            anyhow::bail!(
                "declaring Cairo 0 classes is no longer supported starting \
                from Starkli v0.4.0, as Starknet JSON-RPC v0.8.0 no longer accepts \
                v1 transactions. Use Starkli v0.3.x to declare this class."
            )
        } else if serde_json::from_reader::<_, CompiledClass>(std::fs::File::open(&self.file)?)
            .is_ok()
        {
            // TODO: add more helpful instructions to fix this
            anyhow::bail!("unexpected CASM class");
        } else {
            anyhow::bail!("failed to parse contract artifact");
        };

        let (class_hash, declaration_tx_hash) = match declarable {
            Declarable::CairoOne(mut class) => {
                if self.no_abi {
                    "[]".clone_into(&mut class.abi);
                }

                // Declaring Cairo 1 class
                let class_hash = class.class_hash();

                // TODO: add option to skip checking
                if Self::check_already_declared(&provider, class_hash).await? {
                    return Ok(());
                }

                // Reconstructs an original Sierra class just for CASM compilation purposes. It's a
                // bit inefficient but acceptable.
                let sierra_class = SierraClass {
                    sierra_program: class.sierra_program.clone(),
                    sierra_program_debug_info: SierraClassDebugInfo {
                        type_names: vec![],
                        libfunc_names: vec![],
                        user_func_names: vec![],
                    },
                    contract_class_version: class.contract_class_version.clone(),
                    entry_points_by_type: class.entry_points_by_type.clone(),
                    abi: vec![],
                };

                let casm_source = self.casm.into_casm_hash_source()?;

                if !fee_setting.is_estimate_only() {
                    eprintln!(
                        "Declaring Cairo 1 class: {}",
                        format!("{class_hash:#064x}").bright_yellow()
                    );

                    match &casm_source {
                        CasmHashSource::BuiltInCompiler(_) => {
                            eprintln!(
                                "Compiling Sierra class to CASM with compiler version {}...",
                                format!("{}", BuiltInCompiler::version_for_class(&sierra_class)?)
                                    .bright_yellow()
                            );
                        }
                        CasmHashSource::CompilerBinary(compiler) => {
                            eprintln!(
                                "Compiling Sierra class to CASM with compiler binary {}...",
                                format!("{}", compiler.path().display()).bright_yellow()
                            );
                        }
                        CasmHashSource::CasmFile(path) => {
                            eprintln!(
                                "Using a compiled CASM file directly: {}...",
                                format!("{}", path.display()).bright_yellow()
                            );
                        }
                        CasmHashSource::Hash(hash) => {
                            eprintln!(
                                "Using the provided CASM hash: {}...",
                                format!("{hash:#064x}").bright_yellow()
                            );
                        }
                    }
                }

                let casm_class_hash = casm_source.get_casm_hash(&sierra_class)?;

                if !fee_setting.is_estimate_only() {
                    eprintln!(
                        "CASM class hash: {}",
                        format!("{casm_class_hash:#064x}").bright_yellow()
                    );
                }

                let declare_tx = match fee_setting {
                    FeeSetting::Strk(fee_setting) => {
                        let declaration = account.declare_v3(Arc::new(class), casm_class_hash);
                        let declaration = match self.nonce {
                            Some(nonce) => declaration.nonce(nonce),
                            None => declaration,
                        };

                        let declaration = match fee_setting {
                            TokenFeeSetting::EstimateOnly => {
                                let estimated_fee = declaration
                                    .estimate_fee()
                                    .await
                                    .map_err(account_error_mapper)?
                                    .overall_fee;

                                println!(
                                    "{} STRK",
                                    format!("{}", felt_to_bigdecimal(estimated_fee, 18))
                                        .bright_yellow(),
                                );
                                return Ok(());
                            }
                            TokenFeeSetting::Manual(fee) => {
                                let declaration = if let Some(l1_gas) = fee.l1_gas {
                                    declaration.l1_gas(l1_gas)
                                } else {
                                    declaration
                                };
                                let declaration = if let Some(l2_gas) = fee.l2_gas {
                                    declaration.l2_gas(l2_gas)
                                } else {
                                    declaration
                                };
                                let declaration = if let Some(l1_data_gas) = fee.l1_data_gas {
                                    declaration.l1_data_gas(l1_data_gas)
                                } else {
                                    declaration
                                };

                                let declaration = if let Some(l1_gas_price) = fee.l1_gas_price {
                                    declaration.l1_gas_price(l1_gas_price)
                                } else {
                                    declaration
                                };
                                let declaration = if let Some(l2_gas_price) = fee.l2_gas_price {
                                    declaration.l2_gas_price(l2_gas_price)
                                } else {
                                    declaration
                                };
                                if let Some(l1_data_gas_price) = fee.l1_data_gas_price {
                                    declaration.l1_data_gas_price(l1_data_gas_price)
                                } else {
                                    declaration
                                }
                            }
                            TokenFeeSetting::None => declaration,
                        };

                        if self.simulate {
                            print_colored_json(&declaration.simulate(false, false).await?)?;
                            return Ok(());
                        }

                        declaration.send().await
                    }
                }
                .map_err(account_error_mapper)?
                .transaction_hash;

                (class_hash, declare_tx)
            }
        };

        eprintln!(
            "Contract declaration transaction: {}",
            format!("{declaration_tx_hash:#064x}").bright_yellow()
        );

        if self.watch {
            eprintln!(
                "Waiting for transaction {} to confirm...",
                format!("{declaration_tx_hash:#064x}").bright_yellow(),
            );
            watch_tx(
                &provider,
                declaration_tx_hash,
                Duration::from_millis(self.poll_interval),
            )
            .await?;
        }

        eprintln!("Class hash declared:");

        // Only the class hash goes to stdout so this can be easily scripted
        println!("{}", format!("{class_hash:#064x}").bright_yellow());

        Ok(())
    }

    async fn check_already_declared<P>(provider: P, class_hash: Felt) -> Result<bool>
    where
        P: Provider,
    {
        match provider
            .get_class(BlockId::Tag(BlockTag::Pending), class_hash)
            .await
        {
            Ok(_) => {
                eprintln!("Not declaring class as it's already declared. Class hash:");
                println!("{}", format!("{class_hash:#064x}").bright_yellow());

                Ok(true)
            }
            Err(ProviderError::StarknetError(StarknetError::ClassHashNotFound)) => Ok(false),
            Err(err) => Err(err.into()),
        }
    }
}
