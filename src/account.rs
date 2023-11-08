use std::{fmt::Display, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use starknet::{
    accounts::{ExecutionEncoding, SingleOwnerAccount},
    core::{
        serde::unsigned_field_element::UfeHex,
        types::{BlockId, BlockTag, FieldElement},
        utils::get_contract_address,
    },
    macros::{felt, selector},
    providers::Provider,
    signers::{LocalWallet, SigningKey},
};

use crate::signer::{AnySigner, SignerArgs, SignerResolutionTask};

const BRAAVOS_SIGNER_TYPE_STARK: FieldElement = FieldElement::ONE;

pub const KNOWN_ACCOUNT_CLASSES: [KnownAccountClass; 6] = [
    KnownAccountClass {
        class_hash: felt!("0x048dd59fabc729a5db3afdf649ecaf388e931647ab2f53ca3c6183fa480aa292"),
        variant: AccountVariantType::OpenZeppelinLegacy,
        description: "OpenZeppelin account contract v0.6.1 compiled with cairo-lang v0.11.0.2",
    },
    KnownAccountClass {
        class_hash: felt!("0x04d07e40e93398ed3c76981e72dd1fd22557a78ce36c0515f679e27f0bb5bc5f"),
        variant: AccountVariantType::OpenZeppelinLegacy,
        description: "OpenZeppelin account contract v0.5.0 compiled with cairo-lang v0.10.1",
    },
    KnownAccountClass {
        class_hash: felt!("0x025ec026985a3bf9d0cc1fe17326b245dfdc3ff89b8fde106542a3ea56c5a918"),
        variant: AccountVariantType::ArgentLegacy,
        description: "Argent X legacy (Cairo 0) proxy account",
    },
    KnownAccountClass {
        class_hash: felt!("0x03131fa018d520a037686ce3efddeab8f28895662f019ca3ca18a626650f7d1e"),
        variant: AccountVariantType::Braavos,
        description: "Braavos official proxy account",
    },
    KnownAccountClass {
        class_hash: felt!("0x01a736d6ed154502257f02b1ccdf4d9d1089f80811cd6acad48e6b6a9d1f2003"),
        variant: AccountVariantType::Argent,
        description: "Argent X official account",
    },
    KnownAccountClass {
        class_hash: felt!("0x04c6d6cf894f8bc96bb9c525e6853e5483177841f7388f74a46cfda6f028c755"),
        variant: AccountVariantType::OpenZeppelin,
        description: "OpenZeppelin account contract v0.7.0 compiled with cairo v2.2.0",
    },
];

pub const BUILTIN_ACCOUNTS: &[BuiltinAccount] = &[
    BuiltinAccount {
        id: "katana-0",
        aliases: &["katana0", "katana"],
        address: felt!("0x517ececd29116499f4a1b64b094da79ba08dfd54a3edaa316134c41f8160973"),
        private_key: felt!("0x1800000000300000180000000000030000000000003006001800006600"),
    },
    BuiltinAccount {
        id: "katana-1",
        aliases: &["katana1"],
        address: felt!("0x5686a647a9cdd63ade617e0baf3b364856b813b508f03903eb58a7e622d5855"),
        private_key: felt!("0x33003003001800009900180300d206308b0070db00121318d17b5e6262150b"),
    },
    BuiltinAccount {
        id: "katana-2",
        aliases: &["katana2"],
        address: felt!("0x765149d6bc63271df7b0316537888b81aa021523f9516a05306f10fd36914da"),
        private_key: felt!("0x1c9053c053edf324aec366a34c6901b1095b07af69495bffec7d7fe21effb1b"),
    },
    BuiltinAccount {
        id: "katana-3",
        aliases: &["katana3"],
        address: felt!("0x5006399928dad095cc39818cae15dc022592d47d883701e7814c9db19e96efc"),
        private_key: felt!("0x736adbbcdac7cc600f89051db1abbc16b9996b46f6b58a9752a11c1028a8ec8"),
    },
    BuiltinAccount {
        id: "katana-4",
        aliases: &["katana4"],
        address: felt!("0x586364c42cf7f6c968172ba0806b7403c567544266821c8cd28c292a08b2346"),
        private_key: felt!("0x2bbf4f9fd0bbb2e60b0316c1fe0b76cf7a4d0198bd493ced9b8df2a3a24d68a"),
    },
    BuiltinAccount {
        id: "katana-5",
        aliases: &["katana5"],
        address: felt!("0x70038d685013781745f0ac6fe44f23465f9c55a836fceb119b0d7d379f21026"),
        private_key: felt!("0x283d1e73776cd4ac1ac5f0b879f561bded25eceb2cc589c674af0cec41df441"),
    },
    BuiltinAccount {
        id: "katana-6",
        aliases: &["katana6"],
        address: felt!("0x5ae5b8925c1568f3ec6ab5f4d4ea4b5467e6d6a18f0944608a0d368ac15bdc7"),
        private_key: felt!("0x3e3979c1ed728490308054fe357a9f49cf67f80f9721f44cc57235129e090f4"),
    },
    BuiltinAccount {
        id: "katana-7",
        aliases: &["katana7"],
        address: felt!("0x456b9e6dbbfbfc59e23a80e5ff5ecc59bc02c3c5b9c78ab667471f52c018e87"),
        private_key: felt!("0x6bf3604bcb41fed6c42bcca5436eeb65083a982ff65db0dc123f65358008b51"),
    },
    BuiltinAccount {
        id: "katana-8",
        aliases: &["katana8"],
        address: felt!("0x5c47b38f788ec9d382b5079165bc96c0f49647250199a78d34c436d54d12217"),
        private_key: felt!("0x14d6672dcb4b77ca36a887e9a11cd9d637d5012468175829e9c6e770c61642"),
    },
    BuiltinAccount {
        id: "katana-9",
        aliases: &["katana9"],
        address: felt!("0x74bfdb5562f91764fddbbf3f4fb322de114a00d6d6889b19a4dd7b45d5ba24d"),
        private_key: felt!("0xc5b2fcab997346f3ea1c00b002ecf6f382c5f9c9659a3894eb783c5320f912"),
    },
];

#[derive(Debug, Clone, Parser)]
pub struct AccountArgs {
    #[clap(
        long,
        env = "STARKNET_ACCOUNT",
        help = "Path to account config JSON file"
    )]
    account: String,
    #[clap(flatten)]
    signer: SignerArgs,
}

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
    Braavos(BraavosAccountConfig),
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

// All built-in accounts are assumed to be legacy OZ account for now.
pub struct BuiltinAccount {
    pub id: &'static str,
    pub aliases: &'static [&'static str],
    pub address: FieldElement,
    pub private_key: FieldElement,
}

pub enum AccountVariantType {
    OpenZeppelinLegacy,
    ArgentLegacy,
    Braavos,
    Argent,
    OpenZeppelin,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct OzAccountConfig {
    pub version: u64,
    #[serde_as(as = "UfeHex")]
    pub public_key: FieldElement,
    #[serde(default = "true_as_default")]
    pub legacy: bool,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct ArgentAccountConfig {
    pub version: u64,
    #[serde_as(as = "Option<UfeHex>")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub implementation: Option<FieldElement>,
    #[serde_as(as = "UfeHex")]
    // Old account files use `signer`
    #[serde(alias = "signer")]
    pub owner: FieldElement,
    #[serde_as(as = "UfeHex")]
    pub guardian: FieldElement,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct BraavosAccountConfig {
    pub version: u64,
    #[serde_as(as = "UfeHex")]
    pub implementation: FieldElement,
    pub multisig: BraavosMultisigConfig,
    pub signers: Vec<BraavosSigner>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum BraavosMultisigConfig {
    On { num_signers: usize },
    Off,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BraavosSigner {
    Stark(BraavosStarkSigner),
    // TODO: add secp256r1
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct BraavosStarkSigner {
    #[serde_as(as = "UfeHex")]
    pub public_key: FieldElement,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct UndeployedStatus {
    #[serde_as(as = "UfeHex")]
    pub class_hash: FieldElement,
    #[serde_as(as = "UfeHex")]
    pub salt: FieldElement,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<DeploymentContext>,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct DeployedStatus {
    #[serde_as(as = "UfeHex")]
    pub class_hash: FieldElement,
    #[serde_as(as = "UfeHex")]
    pub address: FieldElement,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "variant", rename_all = "snake_case")]
pub enum DeploymentContext {
    Braavos(BraavosDeploymentContext),
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct BraavosDeploymentContext {
    #[serde_as(as = "UfeHex")]
    pub mock_implementation: FieldElement,
}

impl AccountArgs {
    pub async fn into_account<P>(self, provider: P) -> Result<SingleOwnerAccount<P, AnySigner>>
    where
        P: Provider + Send + Sync,
    {
        let signer = self.signer.into_task()?;

        let mut account = if let Some(builtin_account) = find_builtin_account(&self.account) {
            if matches!(signer, SignerResolutionTask::Strong(_)) {
                // The user is supplying a signer explicitly when using a built-in account. This
                // might be legitimate if the built-in account key has been modified, but it's more
                // likely a user error. We would simply reject it here. Advanced users can always
                // fetch the account into a file and use from there anyways.
                anyhow::bail!(
                    "do not supply signer options when using a built-in account ({})",
                    builtin_account.id
                );
            }

            let chain_id = provider.chain_id().await?;

            SingleOwnerAccount::new(
                provider,
                AnySigner::LocalWallet(LocalWallet::from_signing_key(
                    SigningKey::from_secret_scalar(builtin_account.private_key),
                )),
                builtin_account.address,
                chain_id,
                ExecutionEncoding::Legacy,
            )
        } else {
            let signer = signer.resolve()?;
            let account = PathBuf::from(shellexpand::tilde(&self.account).into_owned());

            if !account.exists() {
                anyhow::bail!("account config file not found");
            }

            let account_config: AccountConfig =
                serde_json::from_reader(&mut std::fs::File::open(&self.account)?)?;

            let account_address = match account_config.deployment {
                DeploymentStatus::Undeployed(_) => anyhow::bail!("account not deployed"),
                DeploymentStatus::Deployed(inner) => inner.address,
            };

            let chain_id = provider.chain_id().await?;

            SingleOwnerAccount::new(
                provider,
                signer,
                account_address,
                chain_id,
                account_config.variant.execution_encoding(),
            )
        };

        account.set_block_id(BlockId::Tag(BlockTag::Pending));

        Ok(account)
    }
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
            AccountVariant::Argent(argent) => match argent.implementation {
                Some(implementation) => {
                    // Legacy Cairo 0 account deployment
                    Ok(get_contract_address(
                        undeployed_status.salt,
                        undeployed_status.class_hash,
                        &[
                            implementation,          // implementation
                            selector!("initialize"), // selector
                            FieldElement::TWO,       // calldata_len
                            argent.owner,            // calldata[0]: signer
                            argent.guardian,         // calldata[1]: guardian
                        ],
                        FieldElement::ZERO,
                    ))
                }
                None => {
                    // Cairo 1 account deployment without using proxy
                    Ok(get_contract_address(
                        undeployed_status.salt,
                        undeployed_status.class_hash,
                        &[argent.owner, argent.guardian],
                        FieldElement::ZERO,
                    ))
                }
            },

            AccountVariant::Braavos(braavos) => {
                if !matches!(braavos.multisig, BraavosMultisigConfig::Off) {
                    anyhow::bail!("Braavos accounts cannot be deployed with multisig on");
                }
                if braavos.signers.len() != 1 {
                    anyhow::bail!("Braavos accounts can only be deployed with one seed signer");
                }

                match &undeployed_status.context {
                    Some(DeploymentContext::Braavos(context)) => {
                        // Safe to unwrap as we already checked for length
                        match braavos.signers.get(0).unwrap() {
                            BraavosSigner::Stark(stark_signer) => {
                                Ok(get_contract_address(
                                    undeployed_status.salt,
                                    undeployed_status.class_hash,
                                    &[
                                        context.mock_implementation, // implementation_address
                                        selector!("initializer"),    // initializer_selector
                                        FieldElement::ONE,           // calldata_len
                                        stark_signer.public_key,     // calldata[0]: public_key
                                    ],
                                    FieldElement::ZERO,
                                ))
                            } // Reject other variants as we add more types
                        }
                    }
                    _ => Err(anyhow::anyhow!("missing Braavos deployment context")),
                }
            }
        }
    }
}

impl AccountVariant {
    pub fn execution_encoding(&self) -> ExecutionEncoding {
        match self {
            AccountVariant::OpenZeppelin(oz) => {
                if oz.legacy {
                    ExecutionEncoding::Legacy
                } else {
                    ExecutionEncoding::New
                }
            }
            AccountVariant::Argent(argent) => {
                if argent.implementation.is_some() {
                    ExecutionEncoding::Legacy
                } else {
                    ExecutionEncoding::New
                }
            }
            AccountVariant::Braavos(_) => ExecutionEncoding::Legacy,
        }
    }
}

impl BraavosSigner {
    pub fn decode(raw_signer_model: &[FieldElement]) -> Result<Self> {
        let raw_signer_type = raw_signer_model
            .get(4)
            .ok_or_else(|| anyhow::anyhow!("unable to read `type` field"))?;

        if raw_signer_type == &BRAAVOS_SIGNER_TYPE_STARK {
            // Index access is safe as we already checked getting the element after
            let public_key = raw_signer_model[0];

            Ok(Self::Stark(BraavosStarkSigner { public_key }))
        } else {
            Err(anyhow::anyhow!("unknown signer type: {}", raw_signer_type))
        }
    }
}

impl Display for AccountVariantType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountVariantType::OpenZeppelinLegacy => write!(f, "Legacy OpenZeppelin (Cairo 0)"),
            AccountVariantType::ArgentLegacy => write!(f, "Legacy Argent X (Cairo 0)"),
            AccountVariantType::Braavos => write!(f, "Braavos"),
            AccountVariantType::Argent => write!(f, "Argent X"),
            AccountVariantType::OpenZeppelin => write!(f, "OpenZeppelin"),
        }
    }
}

fn find_builtin_account(id: &str) -> Option<&'static BuiltinAccount> {
    BUILTIN_ACCOUNTS
        .iter()
        .find(|&account| account.id == id || account.aliases.iter().any(|alias| *alias == id))
}

fn true_as_default() -> bool {
    true
}
