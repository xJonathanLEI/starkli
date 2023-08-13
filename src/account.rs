use std::fmt::Display;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use starknet::{
    core::{
        serde::unsigned_field_element::UfeHex, types::FieldElement, utils::get_contract_address,
    },
    macros::{felt, selector},
};

pub const KNOWN_ACCOUNT_CLASSES: [KnownAccountClass; 2] = [
    KnownAccountClass {
        class_hash: felt!("0x048dd59fabc729a5db3afdf649ecaf388e931647ab2f53ca3c6183fa480aa292"),
        variant: AccountVariantType::OpenZeppelin,
        description: "OpenZeppelin account contract v0.6.1 compiled with cairo-lang v0.11.0.2",
    },
    KnownAccountClass {
        class_hash: felt!("0x025ec026985a3bf9d0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a918"),
        variant: AccountVariantType::Argent,
        description: "Argent X official proxy account",
    },
];

#[derive(Serialize, Deserialize)]
pub struct AccountConfig {
    pub version: u64,
    pub variant: AccountVariant,
    pub deployment: DeploymentStatus,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AccountVariant {
    OpenZeppelin(OzAccountConfig),
    Argent(ArgentAccountConfig),
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum DeploymentStatus {
    Undeployed(UndeployedStatus),
    Deployed(DeployedStatus),
}

pub struct KnownAccountClass {
    pub class_hash: FieldElement,
    pub variant: AccountVariantType,
    pub description: &'static str,
}

pub enum AccountVariantType {
    OpenZeppelin,
    Argent,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct OzAccountConfig {
    pub version: u64,
    #[serde_as(as = "UfeHex")]
    pub public_key: FieldElement,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct ArgentAccountConfig {
    pub version: u64,
    #[serde_as(as = "UfeHex")]
    pub implementation: FieldElement,
    #[serde_as(as = "UfeHex")]
    pub signer: FieldElement,
    #[serde_as(as = "UfeHex")]
    pub guardian: FieldElement,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct UndeployedStatus {
    #[serde_as(as = "UfeHex")]
    pub class_hash: FieldElement,
    #[serde_as(as = "UfeHex")]
    pub salt: FieldElement,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct DeployedStatus {
    #[serde_as(as = "UfeHex")]
    pub class_hash: FieldElement,
    #[serde_as(as = "UfeHex")]
    pub address: FieldElement,
}

impl AccountConfig {
    pub fn deploy_account_address(&self) -> Result<FieldElement> {
        let undeployed_status = match &self.deployment {
            DeploymentStatus::Undeployed(value) => value,
            DeploymentStatus::Deployed(_) => {
                anyhow::bail!("account already deployed");
            }
        };

        match &self.variant {
            AccountVariant::OpenZeppelin(oz) => Ok(get_contract_address(
                undeployed_status.salt,
                undeployed_status.class_hash,
                &[oz.public_key],
                FieldElement::ZERO,
            )),
            AccountVariant::Argent(argent) => Ok(get_contract_address(
                undeployed_status.salt,
                undeployed_status.class_hash,
                &[
                    argent.implementation,   // implementation
                    selector!("initialize"), // selector
                    FieldElement::TWO,       // calldata_len
                    argent.signer,           // calldata[0]: signer
                    argent.guardian,         // calldata[1]: guardian
                ],
                FieldElement::ZERO,
            )),
        }
    }
}

impl Display for AccountVariantType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountVariantType::OpenZeppelin => write!(f, "OpenZeppelin"),
            AccountVariantType::Argent => write!(f, "Argent X"),
        }
    }
}
