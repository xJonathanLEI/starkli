use std::{fmt::Display, str::FromStr};

use anyhow::Result;
use async_trait::async_trait;
use auto_impl::auto_impl;
use clap::{builder::PossibleValue, ValueEnum};
use starknet::providers::Provider;

use crate::provider::ExtendedProvider;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Goerli,
    Integration,
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[auto_impl(&, Box, Arc)]
pub trait NetworkSource {
    async fn get_network(&self) -> Result<Option<Network>>;
}

impl ValueEnum for Network {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Mainnet, Self::Goerli, Self::Integration]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            Network::Mainnet => Some(PossibleValue::new("mainnet").aliases(["alpha-mainnet"])),
            Network::Goerli => Some(PossibleValue::new("goerli-1").aliases([
                "goerli",
                "goerli1",
                "alpha-goerli",
                "alpha-goerli1",
                "alpha-goerli-1",
            ])),
            Network::Integration => Some(PossibleValue::new("integration")),
        }
    }
}

impl FromStr for Network {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "mainnet" | "alpha-mainnet" => Ok(Self::Mainnet),
            "goerli" | "goerli1" | "goerli-1" | "alpha-goerli" | "alpha-goerli1"
            | "alpha-goerli-1" => Ok(Self::Goerli),
            "integration" => Ok(Self::Integration),
            _ => Err(anyhow::anyhow!("unknown network: {}", s)),
        }
    }
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mainnet => write!(f, "mainnet"),
            Self::Goerli => write!(f, "goerli"),
            Self::Integration => write!(f, "integration"),
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl NetworkSource for ExtendedProvider {
    async fn get_network(&self) -> Result<Option<Network>> {
        let chain_id = self.chain_id().await?;
        let is_integration = self.is_integration();

        Ok(if is_integration {
            if chain_id == starknet::core::chain_id::TESTNET {
                Some(Network::Integration)
            } else {
                None
            }
        } else if chain_id == starknet::core::chain_id::MAINNET {
            Some(Network::Mainnet)
        } else if chain_id == starknet::core::chain_id::TESTNET {
            Some(Network::Goerli)
        } else {
            None
        })
    }
}
