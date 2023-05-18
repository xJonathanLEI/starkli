use std::str::FromStr;

use anyhow::Result;
use clap::{builder::PossibleValue, Parser, ValueEnum};
use colored::Colorize;
use starknet::providers::{
    jsonrpc::HttpTransport, AnyProvider, JsonRpcClient, SequencerGatewayProvider,
};
use url::Url;

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

#[derive(Debug, Clone)]
pub enum Network {
    Mainnet,
    Goerli1,
    Goerli2,
}

impl ProviderArgs {
    pub fn into_provider(self) -> AnyProvider {
        match (self.rpc, self.network) {
            (Some(rpc), None) => {
                AnyProvider::JsonRpcHttp(JsonRpcClient::new(HttpTransport::new(rpc)))
            }
            (Some(rpc), Some(_)) => {
                eprintln!(
                    "{}",
                    "WARNING: when using JSON-RPC, the --network flag is ignored. \
                    There's no need to use --network as network is automatically detected."
                        .bright_magenta()
                );

                AnyProvider::JsonRpcHttp(JsonRpcClient::new(HttpTransport::new(rpc)))
            }
            (None, Some(network)) => {
                eprintln!(
                    "{}",
                    "WARNING: you're using --network instead of providing a JSON-RPC endpoint. \
                    Falling back to using the sequencer gateway now, \
                    but this is strongly discouraged."
                        .bright_magenta()
                );

                AnyProvider::SequencerGateway(match network {
                    Network::Mainnet => SequencerGatewayProvider::starknet_alpha_mainnet(),
                    Network::Goerli1 => SequencerGatewayProvider::starknet_alpha_goerli(),
                    Network::Goerli2 => SequencerGatewayProvider::starknet_alpha_goerli_2(),
                })
            }
            (None, None) => {
                // If nothing is provided we fall back to using sequencer gateway for goerli-1
                eprintln!(
                    "{}",
                    "WARNING: no valid provider option found. \
                    Falling back to using the sequencer gateway for the goerli-1 network."
                        .bright_magenta()
                );

                AnyProvider::SequencerGateway(SequencerGatewayProvider::starknet_alpha_goerli())
            }
        }
    }
}

impl ValueEnum for Network {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Mainnet, Self::Goerli1, Self::Goerli2]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            Network::Mainnet => Some(PossibleValue::new("mainnet")),
            Network::Goerli1 => Some(PossibleValue::new("goerli-1")),
            Network::Goerli2 => Some(PossibleValue::new("goerli-2")),
        }
    }
}

impl FromStr for Network {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "mainnet" | "alpha-mainnet" => Ok(Self::Mainnet),
            "goerli" | "goerli1" | "goerli-1" | "alpha-goerli" | "alpha-goerli1"
            | "alpha-goerli-1" => Ok(Self::Goerli1),
            "goerli2" | "goerli-2" | "alpha-goerli2" | "alpha-goerli-2" => Ok(Self::Goerli2),
            _ => Err(anyhow::anyhow!("unknown network: {}", s)),
        }
    }
}
