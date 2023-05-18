use clap::Parser;
use starknet::providers::{jsonrpc::HttpTransport, AnyProvider, JsonRpcClient};
use url::Url;

#[derive(Debug, Clone, Parser)]
pub struct ProviderArgs {
    #[clap(
        long = "rpc",
        env = "STARKNET_RPC",
        help = "Starknet JSON-RPC endpoint"
    )]
    rpc: Url,
}

impl ProviderArgs {
    pub fn into_provider(self) -> AnyProvider {
        AnyProvider::JsonRpcHttp(JsonRpcClient::new(HttpTransport::new(self.rpc)))
    }
}
