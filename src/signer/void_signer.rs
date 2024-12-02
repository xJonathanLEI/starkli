use std::convert::Infallible;

use async_trait::async_trait;
use starknet::{
    core::types::Felt,
    signers::{Signer, SignerInteractivityContext, VerifyingKey},
};
use starknet_crypto::Signature;

#[derive(Debug, Default)]
pub struct VoidSigner;

#[async_trait(?Send)]
impl Signer for VoidSigner {
    type GetPublicKeyError = Infallible;
    type SignError = Infallible;

    async fn get_public_key(&self) -> Result<VerifyingKey, Self::GetPublicKeyError> {
        panic!("This signer is not supported on this platform")
    }

    async fn sign_hash(&self, _hash: &Felt) -> Result<Signature, Self::SignError> {
        panic!("This signer is not supported on this platform")
    }

    fn is_interactive(&self, _context: SignerInteractivityContext<'_>) -> bool {
        panic!("This signer is not supported on this platform")
    }
}
