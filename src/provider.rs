use std::str::FromStr;

use anyhow::Result;
use async_trait::async_trait;
use clap::Parser;
use colored::Colorize;
use indexmap::map::Entry;
use rand::{rngs::StdRng, Rng, SeedableRng};
use starknet::{
    core::types::*,
    macros::short_string,
    providers::{jsonrpc::HttpTransport, AnyProvider, JsonRpcClient, Provider, ProviderError},
};
use url::Url;

use crate::{
    network::Network,
    profile::{FreeProviderVendor, NetworkProvider, Profile, Profiles, DEFAULT_PROFILE_NAME},
};

const CHAIN_ID_MAINNET: FieldElement = short_string!("SN_MAIN");
const CHAIN_ID_GOERLI: FieldElement = short_string!("SN_GOERLI");
const CHAIN_ID_SEPOLIA: FieldElement = short_string!("SN_SEPOLIA");

#[derive(Debug, Clone, Parser)]
pub struct ProviderArgs {
    #[clap(
        long = "rpc",
        env = "STARKNET_RPC",
        help = "Starknet JSON-RPC endpoint"
    )]
    rpc: Option<Url>,
    #[clap(long = "network", env = "STARKNET_NETWORK", help = "Starknet network")]
    network: Option<String>,
}

/// We need this because integration network has the same chain ID as `goerli`. We would otherwise
/// has no way of telling them apart. We could generally just ignore this, but it would actually
/// cause issues when deciding what Sierra compiler version to use depending on network, so we still
/// need this.
pub struct ExtendedProvider {
    provider: AnyProvider,
    is_integration: bool,
}

impl ProviderArgs {
    pub fn into_provider(self) -> Result<ExtendedProvider> {
        Ok(match (self.rpc, self.network) {
            (Some(rpc), None) => ExtendedProvider::new(
                AnyProvider::JsonRpcHttp(JsonRpcClient::new(HttpTransport::new(rpc))),
                false,
            ),
            (Some(rpc), Some(_)) => {
                eprintln!(
                    "{}",
                    "WARNING: the --rpc option and the STARKNET_RPC environment variable take \
                    precedence over the --network option and the STARKNET_NETWORK environment \
                    variable. See https://book.starkli.rs/providers for more details."
                        .bright_magenta()
                );

                ExtendedProvider::new(
                    AnyProvider::JsonRpcHttp(JsonRpcClient::new(HttpTransport::new(rpc))),
                    false,
                )
            }
            (None, Some(network)) => Self::resolve_network(&network)?,
            (None, None) => {
                eprintln!(
                    "{}",
                    "WARNING: you're using neither --rpc (STARKNET_RPC) nor --network \
                    (STARKNET_NETWORK). The `goerli` network is used by default. See \
                    https://book.starkli.rs/providers for more details."
                        .bright_magenta()
                );

                Self::resolve_network("goerli")?
            }
        })
    }

    pub fn resolve_network(network: &str) -> Result<ExtendedProvider> {
        // TODO: move lazy profile loading to a higher level context
        let mut profiles = Profiles::load()?;

        // We save the profiles only when changes are made
        let mut made_changes = false;

        // The only profile supported right now is the `default` profile. We create it if it
        // doesn't exist.
        let matched_profile = match profiles.profiles.entry(DEFAULT_PROFILE_NAME.to_owned()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                made_changes = true;

                entry.insert(Profile {
                    networks: Default::default(),
                })
            }
        };

        let matched_network = match matched_profile.networks.get(network) {
            Some(network) => {
                // The network has been configured. We're good to go!
                network
            }
            None => {
                // This network is not configured. Let's check if it's a known built-in network.
                match Network::from_str(network) {
                    Ok(builtin_network) => {
                        // This is a builtin network. Did we resolve to this network via an alias?
                        // If so, it's possible that the canonical name is configured after all.
                        // Note that we're doing this for backward compatibility only. We might
                        // want to display a warning and remove the aliasing in the future.
                        match matched_profile.networks.get(&builtin_network.to_string()) {
                            Some(network) => {
                                // Yes, although the specified name was not configured, the
                                // canonical name is. Simply return the configured network.
                                network
                            }
                            None => {
                                // The network really is not configured. Let's see if we can
                                // configure it. Only networks with a free provider available can
                                // be configured.
                                //
                                // When configuring a network, we choose a free provider randomly
                                // to be as fair as possible. The chosen provider is persisted for
                                // a consistent experience. A notice is printed to stderr notifying
                                // the user the first time this happens for a certain network.

                                fn choose_vendor(builtin_network: &Network) -> FreeProviderVendor {
                                    let chosen_provider = randome_free_provider(&[
                                        FreeProviderVendor::Blast,
                                        FreeProviderVendor::Nethermind,
                                    ]);

                                    eprintln!(
                                        "{}{}{}{}{}",
                                        "NOTE: you're using the `".bright_magenta(),
                                        format!("{}", builtin_network).bright_yellow(),
                                        "` network without specifying an RPC endpoint for the \
                                        first time. A random free RPC vendor has been selected \
                                        for you: "
                                            .bright_magenta(),
                                        format!("{}", chosen_provider).bright_yellow(),
                                        ". This message will only be shown once. See \
                                        https://book.starkli.rs/providers for more details."
                                            .bright_magenta()
                                    );

                                    chosen_provider
                                }

                                let new_network = match builtin_network {
                                    Network::Mainnet => crate::profile::Network {
                                        name: Some("Starknet Mainnet".into()),
                                        chain_id: CHAIN_ID_MAINNET,
                                        is_integration: false,
                                        provider: NetworkProvider::Free(choose_vendor(
                                            &builtin_network,
                                        )),
                                    },
                                    Network::Goerli => crate::profile::Network {
                                        name: Some("Starknet Goerli Testnet".into()),
                                        chain_id: CHAIN_ID_GOERLI,
                                        is_integration: false,
                                        provider: NetworkProvider::Free(choose_vendor(
                                            &builtin_network,
                                        )),
                                    },
                                    Network::Sepolia => crate::profile::Network {
                                        name: Some("Starknet Sepolia Testnet".into()),
                                        chain_id: CHAIN_ID_SEPOLIA,
                                        is_integration: false,
                                        provider: NetworkProvider::Free(choose_vendor(
                                            &builtin_network,
                                        )),
                                    },
                                    Network::GoerliIntegration | Network::SepoliaIntegration => {
                                        anyhow::bail!(
                                            "network {} cannot be used without being configured",
                                            network
                                        );
                                    }
                                };

                                made_changes = true;

                                matched_profile
                                    .networks
                                    .insert(network.to_owned(), new_network);

                                // We just inserted this so it must exist
                                matched_profile.networks.get(network).unwrap()
                            }
                        }
                    }
                    Err(_) => {
                        anyhow::bail!(
                            "network `{}` is not configured in the active profile, and it's not a \
                            well-known network",
                            network
                        );
                    }
                }
            }
        };

        let rpc_url = match &matched_network.provider {
            NetworkProvider::Rpc(rpc) => rpc.to_owned(),
            NetworkProvider::Free(vendor) => {
                let url = match vendor {
                    FreeProviderVendor::Blast => {
                        if matched_network.chain_id == CHAIN_ID_MAINNET {
                            Some("https://starknet-mainnet.public.blastapi.io/rpc/v0_6")
                        } else if matched_network.chain_id == CHAIN_ID_GOERLI {
                            Some("https://starknet-testnet.public.blastapi.io/rpc/v0_6")
                        } else if matched_network.chain_id == CHAIN_ID_SEPOLIA {
                            Some("https://starknet-sepolia.public.blastapi.io/rpc/v0_6")
                        } else {
                            None
                        }
                    }
                    FreeProviderVendor::Nethermind => {
                        if matched_network.chain_id == CHAIN_ID_MAINNET {
                            Some("https://free-rpc.nethermind.io/mainnet-juno/rpc/v0_6")
                        } else if matched_network.chain_id == CHAIN_ID_GOERLI {
                            Some("https://free-rpc.nethermind.io/goerli-juno/rpc/v0_6")
                        } else if matched_network.chain_id == CHAIN_ID_SEPOLIA {
                            Some("https://free-rpc.nethermind.io/sepolia-juno/rpc/v0_6")
                        } else {
                            None
                        }
                    }
                };

                match url {
                    Some(url) => {
                        // All hard-coded URLs above are valid
                        Url::parse(url).unwrap()
                    }
                    None => {
                        anyhow::bail!(
                            "invalid network in profile: chain ID {:#x} is not supported by \
                            vendor {}",
                            matched_network.chain_id,
                            vendor
                        );
                    }
                }
            }
        };

        let provider = ExtendedProvider::new(
            AnyProvider::JsonRpcHttp(JsonRpcClient::new(HttpTransport::new(rpc_url))),
            matched_network.is_integration,
        );

        if made_changes {
            profiles.save()?;
        }

        Ok(provider)
    }
}

impl ExtendedProvider {
    pub fn new(provider: AnyProvider, is_integration: bool) -> Self {
        Self {
            provider,
            is_integration,
        }
    }

    pub fn is_rpc(&self) -> bool {
        matches!(self.provider, AnyProvider::JsonRpcHttp(_))
    }

    pub fn is_integration(&self) -> bool {
        self.is_integration
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl Provider for ExtendedProvider {
    async fn spec_version(&self) -> Result<String, ProviderError> {
        <AnyProvider as Provider>::spec_version(&self.provider).await
    }

    async fn get_block_with_tx_hashes<B>(
        &self,
        block_id: B,
    ) -> Result<MaybePendingBlockWithTxHashes, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
    {
        <AnyProvider as Provider>::get_block_with_tx_hashes(&self.provider, block_id).await
    }

    async fn get_block_with_txs<B>(
        &self,
        block_id: B,
    ) -> Result<MaybePendingBlockWithTxs, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
    {
        <AnyProvider as Provider>::get_block_with_txs(&self.provider, block_id).await
    }

    async fn get_state_update<B>(
        &self,
        block_id: B,
    ) -> Result<MaybePendingStateUpdate, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
    {
        <AnyProvider as Provider>::get_state_update(&self.provider, block_id).await
    }

    async fn get_storage_at<A, K, B>(
        &self,
        contract_address: A,
        key: K,
        block_id: B,
    ) -> Result<FieldElement, ProviderError>
    where
        A: AsRef<FieldElement> + Send + Sync,
        K: AsRef<FieldElement> + Send + Sync,
        B: AsRef<BlockId> + Send + Sync,
    {
        <AnyProvider as Provider>::get_storage_at(&self.provider, contract_address, key, block_id)
            .await
    }

    async fn get_transaction_status<H>(
        &self,
        transaction_hash: H,
    ) -> Result<TransactionStatus, ProviderError>
    where
        H: AsRef<FieldElement> + Send + Sync,
    {
        <AnyProvider as Provider>::get_transaction_status(&self.provider, transaction_hash).await
    }

    async fn get_transaction_by_hash<H>(
        &self,
        transaction_hash: H,
    ) -> Result<Transaction, ProviderError>
    where
        H: AsRef<FieldElement> + Send + Sync,
    {
        <AnyProvider as Provider>::get_transaction_by_hash(&self.provider, transaction_hash).await
    }

    async fn get_transaction_by_block_id_and_index<B>(
        &self,
        block_id: B,
        index: u64,
    ) -> Result<Transaction, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
    {
        <AnyProvider as Provider>::get_transaction_by_block_id_and_index(
            &self.provider,
            block_id,
            index,
        )
        .await
    }

    async fn get_transaction_receipt<H>(
        &self,
        transaction_hash: H,
    ) -> Result<MaybePendingTransactionReceipt, ProviderError>
    where
        H: AsRef<FieldElement> + Send + Sync,
    {
        <AnyProvider as Provider>::get_transaction_receipt(&self.provider, transaction_hash).await
    }

    async fn get_class<B, H>(
        &self,
        block_id: B,
        class_hash: H,
    ) -> Result<ContractClass, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
        H: AsRef<FieldElement> + Send + Sync,
    {
        <AnyProvider as Provider>::get_class(&self.provider, block_id, class_hash).await
    }

    async fn get_class_hash_at<B, A>(
        &self,
        block_id: B,
        contract_address: A,
    ) -> Result<FieldElement, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
        A: AsRef<FieldElement> + Send + Sync,
    {
        <AnyProvider as Provider>::get_class_hash_at(&self.provider, block_id, contract_address)
            .await
    }

    async fn get_class_at<B, A>(
        &self,
        block_id: B,
        contract_address: A,
    ) -> Result<ContractClass, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
        A: AsRef<FieldElement> + Send + Sync,
    {
        <AnyProvider as Provider>::get_class_at(&self.provider, block_id, contract_address).await
    }

    async fn get_block_transaction_count<B>(&self, block_id: B) -> Result<u64, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
    {
        <AnyProvider as Provider>::get_block_transaction_count(&self.provider, block_id).await
    }

    async fn call<R, B>(&self, request: R, block_id: B) -> Result<Vec<FieldElement>, ProviderError>
    where
        R: AsRef<FunctionCall> + Send + Sync,
        B: AsRef<BlockId> + Send + Sync,
    {
        <AnyProvider as Provider>::call(&self.provider, request, block_id).await
    }

    async fn estimate_fee<R, S, B>(
        &self,
        request: R,
        simulation_flags: S,
        block_id: B,
    ) -> Result<Vec<FeeEstimate>, ProviderError>
    where
        R: AsRef<[BroadcastedTransaction]> + Send + Sync,
        S: AsRef<[SimulationFlagForEstimateFee]> + Send + Sync,
        B: AsRef<BlockId> + Send + Sync,
    {
        <AnyProvider as Provider>::estimate_fee(&self.provider, request, simulation_flags, block_id)
            .await
    }

    async fn estimate_message_fee<M, B>(
        &self,
        message: M,
        block_id: B,
    ) -> Result<FeeEstimate, ProviderError>
    where
        M: AsRef<MsgFromL1> + Send + Sync,
        B: AsRef<BlockId> + Send + Sync,
    {
        <AnyProvider as Provider>::estimate_message_fee(&self.provider, message, block_id).await
    }

    async fn block_number(&self) -> Result<u64, ProviderError> {
        <AnyProvider as Provider>::block_number(&self.provider).await
    }

    async fn block_hash_and_number(&self) -> Result<BlockHashAndNumber, ProviderError> {
        <AnyProvider as Provider>::block_hash_and_number(&self.provider).await
    }

    async fn chain_id(&self) -> Result<FieldElement, ProviderError> {
        <AnyProvider as Provider>::chain_id(&self.provider).await
    }

    async fn syncing(&self) -> Result<SyncStatusType, ProviderError> {
        <AnyProvider as Provider>::syncing(&self.provider).await
    }

    async fn get_events(
        &self,
        filter: EventFilter,
        continuation_token: Option<String>,
        chunk_size: u64,
    ) -> Result<EventsPage, ProviderError> {
        <AnyProvider as Provider>::get_events(
            &self.provider,
            filter,
            continuation_token,
            chunk_size,
        )
        .await
    }

    async fn get_nonce<B, A>(
        &self,
        block_id: B,
        contract_address: A,
    ) -> Result<FieldElement, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
        A: AsRef<FieldElement> + Send + Sync,
    {
        <AnyProvider as Provider>::get_nonce(&self.provider, block_id, contract_address).await
    }

    async fn add_invoke_transaction<I>(
        &self,
        invoke_transaction: I,
    ) -> Result<InvokeTransactionResult, ProviderError>
    where
        I: AsRef<BroadcastedInvokeTransaction> + Send + Sync,
    {
        <AnyProvider as Provider>::add_invoke_transaction(&self.provider, invoke_transaction).await
    }

    async fn add_declare_transaction<D>(
        &self,
        declare_transaction: D,
    ) -> Result<DeclareTransactionResult, ProviderError>
    where
        D: AsRef<BroadcastedDeclareTransaction> + Send + Sync,
    {
        <AnyProvider as Provider>::add_declare_transaction(&self.provider, declare_transaction)
            .await
    }

    async fn add_deploy_account_transaction<D>(
        &self,
        deploy_account_transaction: D,
    ) -> Result<DeployAccountTransactionResult, ProviderError>
    where
        D: AsRef<BroadcastedDeployAccountTransaction> + Send + Sync,
    {
        <AnyProvider as Provider>::add_deploy_account_transaction(
            &self.provider,
            deploy_account_transaction,
        )
        .await
    }

    async fn trace_transaction<H>(
        &self,
        transaction_hash: H,
    ) -> Result<TransactionTrace, ProviderError>
    where
        H: AsRef<FieldElement> + Send + Sync,
    {
        <AnyProvider as Provider>::trace_transaction(&self.provider, transaction_hash).await
    }

    async fn simulate_transactions<B, T, S>(
        &self,
        block_id: B,
        transactions: T,
        simulation_flags: S,
    ) -> Result<Vec<SimulatedTransaction>, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
        T: AsRef<[BroadcastedTransaction]> + Send + Sync,
        S: AsRef<[SimulationFlag]> + Send + Sync,
    {
        <AnyProvider as Provider>::simulate_transactions(
            &self.provider,
            block_id,
            transactions,
            simulation_flags,
        )
        .await
    }

    async fn trace_block_transactions<B>(
        &self,
        block_id: B,
    ) -> Result<Vec<TransactionTraceWithHash>, ProviderError>
    where
        B: AsRef<BlockId> + Send + Sync,
    {
        <AnyProvider as Provider>::trace_block_transactions(&self.provider, block_id).await
    }
}

fn randome_free_provider(choices: &[FreeProviderVendor]) -> FreeProviderVendor {
    let mut rng = StdRng::from_entropy();

    // We never call this function with an empty slice
    let index = rng.gen_range(0..choices.len());
    choices[index]
}
