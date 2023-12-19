use std::{fmt::Display, str::FromStr};

use anyhow::Result;
use async_trait::async_trait;
use auto_impl::auto_impl;
use clap::{builder::PossibleValue, ValueEnum};
use starknet::{macros::short_string, providers::Provider};

use crate::provider::ExtendedProvider;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Goerli,
    Sepolia,
    GoerliIntegration,
    SepoliaIntegration,
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[auto_impl(&, Box, Arc)]
pub trait NetworkSource {
    async fn get_network(&self) -> Result<Option<Network>>;
}

impl ValueEnum for Network {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Self::Mainnet,
            Self::Goerli,
            Self::Sepolia,
            Self::GoerliIntegration,
            Self::SepoliaIntegration,
        ]
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
            Network::Sepolia => {
                Some(PossibleValue::new("sepolia").aliases(["alpha-sepolia", "sepolia-testnet"]))
            }
            Network::GoerliIntegration => {
                Some(PossibleValue::new("goerli-integration").aliases(["integration"]))
            }
            Network::SepoliaIntegration => Some(PossibleValue::new("sepolia-integration")),
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
            "sepolia" | "alpha-sepolia" | "sepolia-testnet" => Ok(Self::Sepolia),
            "goerli-integration" | "integration" => Ok(Self::GoerliIntegration),
            "sepolia-integration" => Ok(Self::SepoliaIntegration),
            _ => Err(anyhow::anyhow!("unknown network: {}", s)),
        }
    }
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mainnet => write!(f, "mainnet"),
            Self::Goerli => write!(f, "goerli"),
            Self::Sepolia => write!(f, "sepolia"),
            Self::GoerliIntegration => write!(f, "goerli-integration"),
            Self::SepoliaIntegration => write!(f, "sepolia-integration"),
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl NetworkSource for ExtendedProvider {
    async fn get_network(&self) -> Result<Option<Network>> {
        let chain_id = self.chain_id().await?;
        let is_integration = self.is_integration();

        Ok(if chain_id == starknet::core::chain_id::MAINNET {
            Some(Network::Mainnet)
        } else if chain_id == starknet::core::chain_id::TESTNET {
            if is_integration {
                Some(Network::GoerliIntegration)
            } else {
                Some(Network::Goerli)
            }
        } else if chain_id == short_string!("SN_SEPOLIA") {
            Some(Network::Sepolia)
        } else if chain_id == short_string!("SN_INTEGRATION_SEPOLIA") {
            Some(Network::SepoliaIntegration)
        } else {
            None
        })
    }
}
