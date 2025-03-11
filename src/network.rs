use std::{fmt::Display, str::FromStr};

use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Sepolia,
    SepoliaIntegration,
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
