use std::{fmt::Display, str::FromStr};

use anyhow::Result;
use async_trait::async_trait;
use auto_impl::auto_impl;
use starknet::{macros::short_string, providers::Provider};

use crate::provider::ExtendedProvider;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Sepolia,
    SepoliaIntegration,
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[auto_impl(&, Box, Arc)]
pub trait NetworkSource {
    async fn get_network(&self) -> Result<Option<Network>>;
}

impl FromStr for Network {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "mainnet" | "alpha-mainnet" => Ok(Self::Mainnet),
            "sepolia" | "alpha-sepolia" | "sepolia-testnet" => Ok(Self::Sepolia),
            "sepolia-integration" => Ok(Self::SepoliaIntegration),
            _ => Err(anyhow::anyhow!("unknown network: {}", s)),
        }
    }
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mainnet => write!(f, "mainnet"),
            Self::Sepolia => write!(f, "sepolia"),
            Self::SepoliaIntegration => write!(f, "sepolia-integration"),
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl NetworkSource for ExtendedProvider {
    async fn get_network(&self) -> Result<Option<Network>> {
        let chain_id = self.chain_id().await?;

        Ok(if chain_id == starknet::core::chain_id::MAINNET {
            Some(Network::Mainnet)
        } else if chain_id == short_string!("SN_SEPOLIA") {
            Some(Network::Sepolia)
        } else if chain_id == short_string!("SN_INTEGRATION_SEPOLIA") {
            Some(Network::SepoliaIntegration)
        } else {
            None
        })
    }
}
