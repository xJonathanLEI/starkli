use async_trait::async_trait;
use starknet::{
    accounts::{
        AccountFactory, ArgentAccountFactory, OpenZeppelinAccountFactory, RawAccountDeployment,
    },
    core::types::FieldElement,
    providers::Provider,
    signers::Signer,
};

mod braavos;
pub use braavos::BraavosAccountFactory;

pub enum AnyAccountFactory<S, P> {
    OpenZeppelin(OpenZeppelinAccountFactory<S, P>),
    Argent(ArgentAccountFactory<S, P>),
    Braavos(BraavosAccountFactory<S, P>),
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<S, P> AccountFactory for AnyAccountFactory<S, P>
where
    S: Signer + Sync + Send,
    P: Provider + Sync + Send,
{
    type Provider = P;
    type SignError = S::SignError;

    fn class_hash(&self) -> FieldElement {
        match self {
            AnyAccountFactory::OpenZeppelin(inner) => inner.class_hash(),
            AnyAccountFactory::Argent(inner) => inner.class_hash(),
            AnyAccountFactory::Braavos(inner) => inner.class_hash(),
        }
    }

    fn calldata(&self) -> Vec<FieldElement> {
        match self {
            AnyAccountFactory::OpenZeppelin(inner) => inner.calldata(),
            AnyAccountFactory::Argent(inner) => inner.calldata(),
            AnyAccountFactory::Braavos(inner) => inner.calldata(),
        }
    }

    fn chain_id(&self) -> FieldElement {
        match self {
            AnyAccountFactory::OpenZeppelin(inner) => inner.chain_id(),
            AnyAccountFactory::Argent(inner) => inner.chain_id(),
            AnyAccountFactory::Braavos(inner) => inner.chain_id(),
        }
    }

    fn provider(&self) -> &Self::Provider {
        match self {
            AnyAccountFactory::OpenZeppelin(inner) => inner.provider(),
            AnyAccountFactory::Argent(inner) => inner.provider(),
            AnyAccountFactory::Braavos(inner) => inner.provider(),
        }
    }

    async fn sign_deployment(
        &self,
        deployment: &RawAccountDeployment,
    ) -> Result<Vec<FieldElement>, Self::SignError> {
        match self {
            AnyAccountFactory::OpenZeppelin(inner) => inner.sign_deployment(deployment).await,
            AnyAccountFactory::Argent(inner) => inner.sign_deployment(deployment).await,
            AnyAccountFactory::Braavos(inner) => inner.sign_deployment(deployment).await,
        }
    }
}
