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
    keystore: Option<PathBuf>,
    #[clap(
        long,
        help = "Supply keystore password from command line option instead of prompt"
    )]
    keystore_password: Option<String>,
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
        match (self.keystore, self.keystore_password) {
            (Some(keystore), keystore_password) => {
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
            _ => Err(anyhow::anyhow!("no valid signer option provided")),
        }
    }
}
