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

pub const KNOWN_ACCOUNT_CLASSES: [KnownAccountClass; 7] = [
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
        description: "Braavos official proxy account (legacy)",
    },
    KnownAccountClass {
        class_hash: felt!("0x0553efc3f74409b08e7bc638c32cadbf1d7d9b19b2fdbff649c7ffe186741ecf"),
        variant: AccountVariantType::Braavos,
        description: "Braavos official proxy account (as of v3.33.3)",
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
        address: felt!("0x3ee9e18edc71a6df30ac3aca2e0b02a198fbce19b7480a63a0d71cbd76652e0"),
        private_key: felt!("0x300001800000000300000180000000000030000000000003006001800006600"),
    },
    BuiltinAccount {
        id: "katana-1",
        aliases: &["katana1"],
        address: felt!("0x33c627a3e5213790e246a917770ce23d7e562baa5b4d2917c23b1be6d91961c"),
        private_key: felt!("0x333803103001800039980190300d206608b0070db0012135bd1fb5f6282170b"),
    },
    BuiltinAccount {
        id: "katana-2",
        aliases: &["katana2"],
        address: felt!("0x1d98d835e43b032254ffbef0f150c5606fa9c5c9310b1fae370ab956a7919f5"),
        private_key: felt!("0x7ca856005bee0329def368d34a6711b2d95b09ef9740ebf2c7c7e3b16c1ca9c"),
    },
    BuiltinAccount {
        id: "katana-3",
        aliases: &["katana3"],
        address: felt!("0x697aaeb6fb12665ced647f7efa57c8f466dc3048556dd265e4774c546caa059"),
        private_key: felt!("0x9f6d7a28c0aec0bb42b11600b2fdc4f20042ab6adeac0ca9e6696aabc5bc95"),
    },
    BuiltinAccount {
        id: "katana-4",
        aliases: &["katana4"],
        address: felt!("0x21b8eb1d455d5a1ef836a8dae16bfa61fbf7aaa252384ab4732603d12d684d2"),
        private_key: felt!("0x5d4184feb2ba1aa1274885dd88c8a670a806066dda7684aa562390441224483"),
    },
    BuiltinAccount {
        id: "katana-5",
        aliases: &["katana5"],
        address: felt!("0x18e623c4ee9f3cf93b06784606f5bc1e86070e8ee6459308c9482554e265367"),
        private_key: felt!("0x1c62fa406d5cac0f365e20ae9c365548f793196e40536c8c118130255a0ac54"),
    },
    BuiltinAccount {
        id: "katana-6",
        aliases: &["katana6"],
        address: felt!("0x1a0a8e7c3a71a44d2e43fec473d7517fd4f20c6ea054e33be3f98ef82e449df"),
        private_key: felt!("0x7813a0576f69d6e2e90d6d5d861f029fa34e528ba418ebb8e335dbc1ed18505"),
    },
    BuiltinAccount {
        id: "katana-7",
        aliases: &["katana7"],
        address: felt!("0x6a933941976911cbf6917010aae47ef7a54bb32846a3d890c1985d879807aa"),
        private_key: felt!("0x92f44f50c2fe38cdd00c59a8ab796238982426341f0ee9ebcaa7fd8b1ac939"),
    },
    BuiltinAccount {
        id: "katana-8",
        aliases: &["katana8"],
        address: felt!("0x3c00d7cda80f89cb59c147d897acb1647f9e33228579674afeea08f6f57e418"),
        private_key: felt!("0x4f5adc57e9025a7c5d1424972354fd83ace8b60ff7d46251512b3ea69b81434"),
    },
    BuiltinAccount {
        id: "katana-9",
        aliases: &["katana9"],
        address: felt!("0x4514dd4ce4762369fc108297f45771f5160aeb7c864d5209e5047a48ab90b52"),
        private_key: felt!("0x4929b5202c17d1bf1329e0f3b1deac313252a007cfd925d703e716f790c5726"),
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
