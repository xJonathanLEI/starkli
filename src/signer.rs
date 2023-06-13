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
    #[clap(long, help = "Path to keystore JSON file")]
    keystore: Option<String>,
    #[clap(
        long,
        help = "Supply keystore password from command line option instead of prompt"
    )]
    keystore_password: Option<String>,
    #[clap(long, help = "Private key in hex in plain text")]
    private_key: Option<String>,
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

        match (keystore, self.keystore_password, self.private_key) {
            (Some(StringValue::FromCommandLine(keystore)), keystore_password, None) => {
                Self::resolve_keystore(keystore, keystore_password)
            }
            (None, None, Some(private_key)) => Self::resolve_private_key(private_key),
            (Some(StringValue::FromEnvVar(_)), None, Some(private_key)) => {
                Self::resolve_private_key(private_key)
            }
            (Some(StringValue::FromEnvVar(keystore)), keystore_password, None) => {
                Self::resolve_keystore(keystore, keystore_password)
            }
            _ => Err(anyhow::anyhow!("no valid signer option provided")),
        }
    }

    fn resolve_keystore(keystore: String, keystore_password: Option<String>) -> Result<AnySigner> {
        let keystore = PathBuf::from(&keystore);

        if keystore_password.is_some() {
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

        let password = if let Some(password) = keystore_password {
            password
        } else {
            rpassword::prompt_password("Enter keystore password: ")?
        };

        let key = SigningKey::from_keystore(keystore, &password)?;

        Ok(AnySigner::LocalWallet(LocalWallet::from_signing_key(key)))
    }

    fn resolve_private_key(private_key: String) -> Result<AnySigner> {
        // TODO: change to recommend hardware wallets when they become available
        eprintln!(
            "{}",
            "WARNING: using private key in plain text is highly insecure, and you should \
                    ONLY do this for development. Consider using an encrypted keystore instead."
                .bright_magenta()
        );

        let private_key = FieldElement::from_hex_be(&private_key)?;
        let key = SigningKey::from_secret_scalar(private_key);

        Ok(AnySigner::LocalWallet(LocalWallet::from_signing_key(key)))
    }
}
