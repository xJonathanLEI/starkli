use std::{fmt::Display, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use starknet::{
    accounts::{ExecutionEncoding, SingleOwnerAccount},
    core::{
        serde::unsigned_field_element::UfeHex,
        types::{BlockId, BlockTag, Felt},
        utils::get_contract_address,
    },
    macros::{felt, selector},
    providers::Provider,
    signers::{LocalWallet, SigningKey},
};

use crate::signer::{AnySigner, SignerArgs, SignerResolutionTask};

const BRAAVOS_SIGNER_TYPE_STARK: Felt = Felt::ONE;

pub const KNOWN_ACCOUNT_CLASSES: [KnownAccountClass; 22] = [
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
        variant: AccountVariantType::BraavosLegacy,
        description: "Braavos legacy (Cairo 0) proxy account",
    },
    KnownAccountClass {
        class_hash: felt!("0x0553efc3f74409b08e7bc638c32cadbf1d7d9b19b2fdbff649c7ffe186741ecf"),
        variant: AccountVariantType::BraavosLegacy,
        description: "Braavos legacy (Cairo 0) proxy account (as of v3.33.3)",
    },
    KnownAccountClass {
        class_hash: felt!("0x00816dd0297efc55dc1e7559020a3a825e81ef734b558f03c83325d4da7e6253"),
        variant: AccountVariantType::Braavos,
        description: "Braavos official account (as of v3.37.4)",
    },
    KnownAccountClass {
        class_hash: felt!("0x01a736d6ed154502257f02b1ccdf4d9d1089f80811cd6acad48e6b6a9d1f2003"),
        variant: AccountVariantType::Argent,
        description: "Argent X official account (as of 5.7.0)",
    },
    KnownAccountClass {
        class_hash: felt!("0x029927c8af6bccf3f6fda035981e765a7bdbf18a2dc0d630494f8758aa908e2b"),
        variant: AccountVariantType::Argent,
        description: "Argent X official account (as of 5.13.1)",
    },
    KnownAccountClass {
        class_hash: felt!("0x036078334509b514626504edc9fb252328d1a240e4e948bef8d0c08dff45927f"),
        variant: AccountVariantType::Argent,
        description: "Argent X official account (as of 5.16.3) compiled with cairo v2.6.3",
    },
    KnownAccountClass {
        class_hash: felt!("0x04c6d6cf894f8bc96bb9c525e6853e5483177841f7388f74a46cfda6f028c755"),
        variant: AccountVariantType::OpenZeppelin,
        description: "OpenZeppelin account contract v0.7.0 compiled with cairo v2.2.0",
    },
    KnownAccountClass {
        class_hash: felt!("0x05400e90f7e0ae78bd02c77cd75527280470e2fe19c54970dd79dc37a9d3645c"),
        variant: AccountVariantType::OpenZeppelin,
        description: "OpenZeppelin account contract v0.8.0 compiled with cairo v2.3.1",
    },
    KnownAccountClass {
        class_hash: felt!("0x061dac032f228abef9c6626f995015233097ae253a7f72d68552db02f2971b8f"),
        variant: AccountVariantType::OpenZeppelin,
        description: "OpenZeppelin account contract v0.8.1 compiled with cairo v2.4.1",
    },
    KnownAccountClass {
        class_hash: felt!("0x01148c31dfa5c4708a4e9cf1eb0fd3d4d8ad9ccf09d0232cd6b56bee64a7de9d"),
        variant: AccountVariantType::OpenZeppelin,
        description: "OpenZeppelin account contract v0.9.0 compiled with cairo v2.5.3",
    },
    KnownAccountClass {
        class_hash: felt!("0x004ca5c0b1af6115858708bd1fabd2e9bb306012b31a741acbf69b8a9f35d688"),
        variant: AccountVariantType::OpenZeppelin,
        description: "OpenZeppelin account contract v0.10.0 compiled with cairo v2.5.3",
    },
    KnownAccountClass {
        class_hash: felt!("0x0450f568a8cb6ea1bcce446355e8a1c2e5852a6b8dc3536f495cdceb62e8a7e2"),
        variant: AccountVariantType::OpenZeppelin,
        description: "OpenZeppelin account contract v0.11.0 compiled with cairo v2.6.3",
    },
    KnownAccountClass {
        class_hash: felt!("0x01e60c8722677cfb7dd8dbea5be86c09265db02cdfe77113e77da7d44c017388"),
        variant: AccountVariantType::OpenZeppelin,
        description: "OpenZeppelin account contract v0.12.0 compiled with cairo v2.6.3",
    },
    KnownAccountClass {
        class_hash: felt!("0x00e2eb8f5672af4e6a4e8a8f1b44989685e668489b0a25437733756c5a34a1d6"),
        variant: AccountVariantType::OpenZeppelin,
        description: "OpenZeppelin account contract v0.13.0 compiled with cairo v2.6.3",
    },
    KnownAccountClass {
        class_hash: felt!("0x04343194a4a6082192502e132d9e7834b5d9bfc7a0c1dd990e95b66f85a66d46"),
        variant: AccountVariantType::OpenZeppelin,
        description: "OpenZeppelin account contract v0.14.0 compiled with cairo v2.6.4",
    },
    KnownAccountClass {
        class_hash: felt!("0x005b8b908d16497cd347642c1ab43015b5956e2085aa4c8bb21f0b073015a1c9"),
        variant: AccountVariantType::OpenZeppelin,
        // Same hash as v0.15.0
        description: "OpenZeppelin account contract v0.15.1 compiled with cairo v2.7.0",
    },
    KnownAccountClass {
        class_hash: felt!("0x0358a1635f95aaaa840ec3b47219a354d5dfe6b01f0bca38ae6b2ff397490348"),
        variant: AccountVariantType::OpenZeppelin,
        description: "OpenZeppelin account contract v0.16.0 compiled with cairo v2.8.0",
    },
    KnownAccountClass {
        class_hash: felt!("0x06d4b80c0f3c3ea9e98252403a83f8a6bacf7f7362e9ac0a8824854dca31f8a8"),
        variant: AccountVariantType::OpenZeppelin,
        description: "OpenZeppelin account contract v0.17.0 compiled with cairo v2.8.2",
    },
    KnownAccountClass {
        class_hash: felt!("0x04a444ef8caf8fa0db05da60bf0ad9bae264c73fa7e32c61d245406f5523174b"),
        variant: AccountVariantType::OpenZeppelin,
        // Same hash for v0.18.0
        description: "OpenZeppelin account contract v0.19.0 compiled with cairo v2.8.4",
    },
];

pub const BUILTIN_ACCOUNTS: &[BuiltinAccount] = &[
    BuiltinAccount {
        id: "katana-0",
        aliases: &["katana0", "katana"],
        address: felt!("0x127fd5f1fe78a71f8bcd1fec63e3fe2f0486b6ecd5c86a0466c3a21fa5cfcec"),
        private_key: felt!("0xc5b2fcab997346f3ea1c00b002ecf6f382c5f9c9659a3894eb783c5320f912"),
    },
    BuiltinAccount {
        id: "katana-1",
        aliases: &["katana1"],
        address: felt!("0x13d9ee239f33fea4f8785b9e3870ade909e20a9599ae7cd62c1c292b73af1b7"),
        private_key: felt!("0x1c9053c053edf324aec366a34c6901b1095b07af69495bffec7d7fe21effb1b"),
    },
    BuiltinAccount {
        id: "katana-2",
        aliases: &["katana2"],
        address: felt!("0x17cc6ca902ed4e8baa8463a7009ff18cc294fa85a94b4ce6ac30a9ebd6057c7"),
        private_key: felt!("0x14d6672dcb4b77ca36a887e9a11cd9d637d5012468175829e9c6e770c61642"),
    },
    BuiltinAccount {
        id: "katana-3",
        aliases: &["katana3"],
        address: felt!("0x2af9427c5a277474c079a1283c880ee8a6f0f8fbf73ce969c08d88befec1bba"),
        private_key: felt!("0x1800000000300000180000000000030000000000003006001800006600"),
    },
    BuiltinAccount {
        id: "katana-4",
        aliases: &["katana4"],
        address: felt!("0x359b9068eadcaaa449c08b79a367c6fdfba9448c29e96934e3552dab0fdd950"),
        private_key: felt!("0x2bbf4f9fd0bbb2e60b0316c1fe0b76cf7a4d0198bd493ced9b8df2a3a24d68a"),
    },
    BuiltinAccount {
        id: "katana-5",
        aliases: &["katana5"],
        address: felt!("0x4184158a64a82eb982ff702e4041a49db16fa3a18229aac4ce88c832baf56e4"),
        private_key: felt!("0x6bf3604bcb41fed6c42bcca5436eeb65083a982ff65db0dc123f65358008b51"),
    },
    BuiltinAccount {
        id: "katana-6",
        aliases: &["katana6"],
        address: felt!("0x42b249d1633812d903f303d640a4261f58fead5aa24925a9efc1dd9d76fb555"),
        private_key: felt!("0x283d1e73776cd4ac1ac5f0b879f561bded25eceb2cc589c674af0cec41df441"),
    },
    BuiltinAccount {
        id: "katana-7",
        aliases: &["katana7"],
        address: felt!("0x4e0b838810cb1a355beb7b3d894ca0e98ee524309c3f8b7cccb15a48e6270e2"),
        private_key: felt!("0x736adbbcdac7cc600f89051db1abbc16b9996b46f6b58a9752a11c1028a8ec8"),
    },
    BuiltinAccount {
        id: "katana-8",
        aliases: &["katana8"],
        address: felt!("0x5b6b8189bb580f0df1e6d6bec509ff0d6c9be7365d10627e0cf222ec1b47a71"),
        private_key: felt!("0x33003003001800009900180300d206308b0070db00121318d17b5e6262150b"),
    },
    BuiltinAccount {
        id: "katana-9",
        aliases: &["katana9"],
        address: felt!("0x6677fe62ee39c7b07401f754138502bab7fac99d2d3c5d37df7d1c6fab10819"),
        private_key: felt!("0x3e3979c1ed728490308054fe357a9f49cf67f80f9721f44cc57235129e090f4"),
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
    pub class_hash: Felt,
    pub variant: AccountVariantType,
    pub description: &'static str,
}

// All built-in accounts are assumed to be legacy OZ account for now.
pub struct BuiltinAccount {
    pub id: &'static str,
    pub aliases: &'static [&'static str],
    pub address: Felt,
    pub private_key: Felt,
}

pub enum AccountVariantType {
    OpenZeppelinLegacy,
    ArgentLegacy,
    BraavosLegacy,
    Argent,
    Braavos,
    OpenZeppelin,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct OzAccountConfig {
    pub version: u64,
    #[serde_as(as = "UfeHex")]
    pub public_key: Felt,
    #[serde(default = "true_as_default")]
    pub legacy: bool,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct ArgentAccountConfig {
    pub version: u64,
    #[serde_as(as = "Option<UfeHex>")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub implementation: Option<Felt>,
    #[serde_as(as = "UfeHex")]
    // Old account files use `signer`
    #[serde(alias = "signer")]
    pub owner: Felt,
    #[serde_as(as = "UfeHex")]
    pub guardian: Felt,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct BraavosAccountConfig {
    pub version: u64,
    #[serde_as(as = "Option<UfeHex>")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub implementation: Option<Felt>,
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
    pub public_key: Felt,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct UndeployedStatus {
    #[serde_as(as = "UfeHex")]
    pub class_hash: Felt,
    #[serde_as(as = "UfeHex")]
    pub salt: Felt,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<DeploymentContext>,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct DeployedStatus {
    #[serde_as(as = "UfeHex")]
    pub class_hash: Felt,
    #[serde_as(as = "UfeHex")]
    pub address: Felt,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "variant", rename_all = "snake_case")]
pub enum DeploymentContext {
    Braavos(BraavosDeploymentContext),
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct BraavosDeploymentContext {
    // Old account files use `mock_implementation`
    #[serde(alias = "mock_implementation")]
    #[serde_as(as = "UfeHex")]
    pub base_account_class_hash: Felt,
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
                // All built-in accounts are now on Cairo 1
                ExecutionEncoding::New,
            )
        } else {
            let signer = signer.resolve().await?;
            let account = PathBuf::from(shellexpand::tilde(&self.account).into_owned());

            if !account.exists() {
                anyhow::bail!("account config file not found");
            }

            let account_config: AccountConfig =
                serde_json::from_reader(&mut std::fs::File::open(&account)?)?;

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
    pub fn deploy_account_address(&self) -> Result<Felt> {
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
                Felt::ZERO,
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
                            Felt::TWO,               // calldata_len
                            argent.owner,            // calldata[0]: signer
                            argent.guardian,         // calldata[1]: guardian
                        ],
                        Felt::ZERO,
                    ))
                }
                None => {
                    // Cairo 1 account deployment without using proxy

                    // This assumes Argent account contract v0.4.0 or later, as the constructor
                    // signature changed with v0.4.0. This means that trying to deploy older Argent
                    // account files will fail.
                    //
                    // We simply assume that no one still has undeployed old Argent account files to
                    // reduce maintenance burden.
                    let mut calldata = vec![Felt::ZERO, argent.owner];
                    if argent.guardian == Felt::ZERO {
                        // Option::None
                        calldata.push(Felt::ONE);
                    } else {
                        // Option::Some(Signer::StarknetSigner)
                        calldata.push(Felt::ZERO);
                        calldata.push(Felt::ZERO);
                        calldata.push(argent.guardian);
                    }

                    Ok(get_contract_address(
                        undeployed_status.salt,
                        undeployed_status.class_hash,
                        &calldata,
                        Felt::ZERO,
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
                        match braavos.signers.first().unwrap() {
                            BraavosSigner::Stark(stark_signer) => {
                                Ok(get_contract_address(
                                    undeployed_status.salt,
                                    context.base_account_class_hash,
                                    &[
                                        stark_signer.public_key, // calldata[0]: public_key
                                    ],
                                    Felt::ZERO,
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
            AccountVariant::Braavos(braavos) => {
                if braavos.implementation.is_some() {
                    ExecutionEncoding::Legacy
                } else {
                    ExecutionEncoding::New
                }
            }
        }
    }
}

impl BraavosSigner {
    pub fn decode(raw_signer_model: &[Felt]) -> Result<Self> {
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
            AccountVariantType::BraavosLegacy => write!(f, "Legacy Braavos (Cairo 0)"),
            AccountVariantType::Argent => write!(f, "Argent X"),
            AccountVariantType::Braavos => write!(f, "Braavos"),
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
