use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;
use clap::Parser;
use colored::Colorize;
use starknet::{
    core::{crypto::Signature, types::FieldElement},
    signers::{LocalWallet, Signer, SigningKey, VerifyingKey},
};

#[derive(Debug)]
pub enum AnySigner {
    LocalWallet(LocalWallet),
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum AnySignerGetPublicKeyError {
    LocalWallet(<LocalWallet as Signer>::GetPublicKeyError),
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum AnySignerSignError {
    LocalWallet(<LocalWallet as Signer>::SignError),
}

#[derive(Debug, Clone, Parser)]
pub struct SignerArgs {
    #[clap(long, help = keystore_help())]
    keystore: Option<String>,
    #[clap(
        long,
        help = "Supply keystore password from command line option instead of prompt"
    )]
    keystore_password: Option<String>,
    #[clap(long, help = "Private key in hex in plain text")]
    private_key: Option<String>,
}

#[derive(Debug)]
pub enum SignerResolutionTask {
    /// The user explicitly requested to use a signer, usually from the command line.
    Strong(SignerResolutionTaskContent),
    /// The signer comes from a global default or environment variable.
    Weak(SignerResolutionTaskContent),
    /// No signer option is provided at all.
    None,
}

#[derive(Debug)]
pub enum SignerResolutionTaskContent {
    Keystore(KeystoreTaskContent),
    PrivateKey(PrivateKeyTaskContent),
}

#[derive(Debug)]
pub struct KeystoreTaskContent {
    keystore: String,
    keystore_password: Option<String>,
}

#[derive(Debug)]
pub struct PrivateKeyTaskContent {
    key: String,
}

enum StringValue {
    FromCommandLine(String),
    FromEnvVar(String),
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl Signer for AnySigner {
    type GetPublicKeyError = AnySignerGetPublicKeyError;
    type SignError = AnySignerSignError;

    async fn get_public_key(&self) -> Result<VerifyingKey, Self::GetPublicKeyError> {
        match self {
            Self::LocalWallet(inner) => Ok(<LocalWallet as Signer>::get_public_key(inner)
                .await
                .map_err(Self::GetPublicKeyError::LocalWallet)?),
        }
    }

    async fn sign_hash(&self, hash: &FieldElement) -> Result<Signature, Self::SignError> {
        match self {
            Self::LocalWallet(inner) => Ok(<LocalWallet as Signer>::sign_hash(inner, hash)
                .await
                .map_err(Self::SignError::LocalWallet)?),
        }
    }
}

impl SignerArgs {
    pub fn into_signer(self) -> Result<AnySigner> {
        self.into_task()?.resolve()
    }

    /// Parses the options into a resolution task without immediately performing the resolution.
    /// This method allows callers to defer resolution to a later stage while still performing some
    /// initial validations.
    pub fn into_task(self) -> Result<SignerResolutionTask> {
        // We're not using the `env` derive from `clap` because we need to distinguish between
        // whether the value is supplied from the command line or the environment variable.
        //
        // This distinction is important because we would not yell at the user for having option
        // conflicts from env vars. This allows us to reject conflicts on options provided from the
        // command line while ignoring those from env vars.
        let keystore = match self.keystore {
            Some(value) => Some(StringValue::FromCommandLine(value)),
            None => match std::env::var("STARKNET_KEYSTORE") {
                Ok(value) => Some(StringValue::FromEnvVar(value)),
                Err(_) => None,
            },
        };

        let task = match (keystore, self.keystore_password, self.private_key) {
            (Some(StringValue::FromCommandLine(keystore)), keystore_password, None) => {
                SignerResolutionTask::Strong(SignerResolutionTaskContent::Keystore(
                    KeystoreTaskContent {
                        keystore,
                        keystore_password,
                    },
                ))
            }
            (None, None, Some(private_key)) => SignerResolutionTask::Strong(
                SignerResolutionTaskContent::PrivateKey(PrivateKeyTaskContent { key: private_key }),
            ),
            (Some(StringValue::FromEnvVar(_)), None, Some(private_key)) => {
                SignerResolutionTask::Strong(SignerResolutionTaskContent::PrivateKey(
                    PrivateKeyTaskContent { key: private_key },
                ))
            }
            (Some(StringValue::FromEnvVar(keystore)), keystore_password, None) => {
                SignerResolutionTask::Weak(SignerResolutionTaskContent::Keystore(
                    KeystoreTaskContent {
                        keystore,
                        keystore_password,
                    },
                ))
            }
            (None, None, None) => SignerResolutionTask::None,
            _ => {
                return Err(anyhow::anyhow!(
                    "invalid signer option combination. \
                    Do not mix options of different signer sources."
                ))
            }
        };

        Ok(task)
    }
}

impl SignerResolutionTask {
    pub fn resolve(self) -> Result<AnySigner> {
        match self {
            Self::Strong(task) | Self::Weak(task) => match task {
                SignerResolutionTaskContent::Keystore(inner) => inner.resolve(),
                SignerResolutionTaskContent::PrivateKey(inner) => inner.resolve(),
            },
            Self::None => Err(anyhow::anyhow!(
                "no valid signer option provided. \
                Consider using a keystore by providing a --keystore option.\
                \n\nFor more information, see: https://book.starkli.rs/signers"
            )),
        }
    }
}

impl KeystoreTaskContent {
    pub fn resolve(self) -> Result<AnySigner> {
        if self.keystore.is_empty() {
            anyhow::bail!("empty keystore path");
        }

        let keystore = PathBuf::from(shellexpand::tilde(&self.keystore).into_owned());

        if self.keystore_password.is_some() {
            eprintln!(
                "{}",
                "WARNING: setting keystore passwords via --password is generally \
                        considered insecure, as they will be stored in your shell history or other \
                        log files."
                    .bright_magenta()
            );
        }

        if !keystore.exists() {
            anyhow::bail!("keystore file not found");
        }

        let password = if let Some(password) = self.keystore_password {
            password
        } else {
            rpassword::prompt_password("Enter keystore password: ")?
        };

        let key = SigningKey::from_keystore(keystore, &password)?;

        Ok(AnySigner::LocalWallet(LocalWallet::from_signing_key(key)))
    }
}

impl PrivateKeyTaskContent {
    pub fn resolve(self) -> Result<AnySigner> {
        // TODO: change to recommend hardware wallets when they become available
        eprintln!(
            "{}",
            "WARNING: using private key in plain text is highly insecure, and you should \
                    ONLY do this for development. Consider using an encrypted keystore instead."
                .bright_magenta()
        );

        let private_key = FieldElement::from_hex_be(&self.key)?;
        let key = SigningKey::from_secret_scalar(private_key);

        Ok(AnySigner::LocalWallet(LocalWallet::from_signing_key(key)))
    }
}

fn keystore_help() -> String {
    format!(
        "Path to keystore JSON file [env: STARKNET_KEYSTORE={}]",
        std::env::var("STARKNET_KEYSTORE").unwrap_or_default()
    )
}
