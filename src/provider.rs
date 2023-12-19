use anyhow::Result;
use async_trait::async_trait;
use clap::Parser;
use colored::Colorize;
use starknet::{
    core::{chain_id, types::*},
    macros::short_string,
    providers::{
        jsonrpc::HttpTransport, AnyProvider, JsonRpcClient, Provider, ProviderError,
        SequencerGatewayProvider,
    },
};
use url::Url;

use crate::network::Network;

#[derive(Debug, Clone, Parser)]
pub struct ProviderArgs {
    #[clap(
        long = "rpc",
        env = "STARKNET_RPC",
        help = "Starknet JSON-RPC endpoint"
    )]
    rpc: Option<Url>,
    #[clap(long = "network", env = "STARKNET_NETWORK", help = "Starknet network")]
    network: Option<Network>,
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
    pub fn into_provider(self) -> ExtendedProvider {
        match (self.rpc, self.network) {
            (Some(rpc), None) => ExtendedProvider::new(
                AnyProvider::JsonRpcHttp(JsonRpcClient::new(HttpTransport::new(rpc))),
                false,
            ),
            (Some(rpc), Some(_)) => {
                eprintln!(
                    "{}",
                    "WARNING: when using JSON-RPC, the --network option and the STARKNET_NETWORK \
                    environment variable are ignored, as those are for using the deprecated \
                    sequencer gateway. See https://book.starkli.rs/providers for more details."
                        .bright_magenta()
                );

                ExtendedProvider::new(
                    AnyProvider::JsonRpcHttp(JsonRpcClient::new(HttpTransport::new(rpc))),
                    false,
                )
            }
            (None, Some(network)) => {
                eprintln!(
                    "{}",
                    "WARNING: you're using the sequencer gateway instead of providing a JSON-RPC \
                    endpoint. This is strongly discouraged. See https://book.starkli.rs/providers \
                    for more details."
                        .bright_magenta()
                );

                ExtendedProvider::new(
                    AnyProvider::SequencerGateway(match network {
                        Network::Mainnet => SequencerGatewayProvider::starknet_alpha_mainnet(),
                        Network::Goerli => SequencerGatewayProvider::starknet_alpha_goerli(),
                        Network::Sepolia => SequencerGatewayProvider::new(
                            Url::parse("https://alpha-sepolia.starknet.io/gateway").unwrap(),
                            Url::parse("https://alpha-sepolia.starknet.io/feeder_gateway").unwrap(),
                            short_string!("SN_SEPOLIA"),
                        ),
                        Network::Integration => SequencerGatewayProvider::new(
                            Url::parse("https://external.integration.starknet.io/gateway").unwrap(),
                            Url::parse("https://external.integration.starknet.io/feeder_gateway")
                                .unwrap(),
                            chain_id::TESTNET,
                        ),
                    }),
                    match network {
                        Network::Mainnet | Network::Goerli | Network::Sepolia => false,
                        Network::Integration => true,
                    },
                )
            }
            (None, None) => {
                // If nothing is provided we fall back to using sequencer gateway for goerli
                eprintln!(
                    "{}",
                    "WARNING: no valid provider option found. Falling back to using the sequencer \
                    gateway for the goerli network. Doing this is discouraged. See \
                    https://book.starkli.rs/providers for more details."
                        .bright_magenta()
                );

                ExtendedProvider::new(
                    AnyProvider::SequencerGateway(SequencerGatewayProvider::starknet_alpha_goerli()),
                    false,
                )
            }
        }
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

    async fn estimate_fee<R, B>(
        &self,
        request: R,
        block_id: B,
    ) -> Result<Vec<FeeEstimate>, ProviderError>
    where
        R: AsRef<[BroadcastedTransaction]> + Send + Sync,
        B: AsRef<BlockId> + Send + Sync,
    {
        <AnyProvider as Provider>::estimate_fee(&self.provider, request, block_id).await
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

    async fn pending_transactions(&self) -> Result<Vec<Transaction>, ProviderError> {
        <AnyProvider as Provider>::pending_transactions(&self.provider).await
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

    async fn trace_block_transactions<H>(
        &self,
        block_hash: H,
    ) -> Result<Vec<TransactionTraceWithHash>, ProviderError>
    where
        H: AsRef<FieldElement> + Send + Sync,
    {
        <AnyProvider as Provider>::trace_block_transactions(&self.provider, block_hash).await
    }
}
