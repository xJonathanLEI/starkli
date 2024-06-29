use async_trait::async_trait;
use starknet::{
    accounts::{
        AccountFactory, ArgentAccountFactory, OpenZeppelinAccountFactory, RawAccountDeploymentV1,
        RawAccountDeploymentV3,
    },
    core::types::{BlockId, Felt},
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

    fn class_hash(&self) -> Felt {
        match self {
            AnyAccountFactory::OpenZeppelin(inner) => inner.class_hash(),
            AnyAccountFactory::Argent(inner) => inner.class_hash(),
            AnyAccountFactory::Braavos(inner) => inner.class_hash(),
        }
    }

    fn calldata(&self) -> Vec<Felt> {
        match self {
            AnyAccountFactory::OpenZeppelin(inner) => inner.calldata(),
            AnyAccountFactory::Argent(inner) => inner.calldata(),
            AnyAccountFactory::Braavos(inner) => inner.calldata(),
        }
    }

    fn chain_id(&self) -> Felt {
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

    fn is_signer_interactive(&self) -> bool {
        match self {
            AnyAccountFactory::OpenZeppelin(inner) => inner.is_signer_interactive(),
            AnyAccountFactory::Argent(inner) => inner.is_signer_interactive(),
            AnyAccountFactory::Braavos(inner) => inner.is_signer_interactive(),
        }
    }

    fn block_id(&self) -> BlockId {
        match self {
            AnyAccountFactory::OpenZeppelin(inner) => inner.block_id(),
            AnyAccountFactory::Argent(inner) => inner.block_id(),
            AnyAccountFactory::Braavos(inner) => inner.block_id(),
        }
    }

    async fn sign_deployment_v1(
        &self,
        deployment: &RawAccountDeploymentV1,
        query_only: bool,
    ) -> Result<Vec<Felt>, Self::SignError> {
        match self {
            AnyAccountFactory::OpenZeppelin(inner) => {
                inner.sign_deployment_v1(deployment, query_only).await
            }
            AnyAccountFactory::Argent(inner) => {
                inner.sign_deployment_v1(deployment, query_only).await
            }
            AnyAccountFactory::Braavos(inner) => {
                inner.sign_deployment_v1(deployment, query_only).await
            }
        }
    }

    async fn sign_deployment_v3(
        &self,
        deployment: &RawAccountDeploymentV3,
        query_only: bool,
    ) -> Result<Vec<Felt>, Self::SignError> {
        match self {
            AnyAccountFactory::OpenZeppelin(inner) => {
                inner.sign_deployment_v3(deployment, query_only).await
            }
            AnyAccountFactory::Argent(inner) => {
                inner.sign_deployment_v3(deployment, query_only).await
            }
            AnyAccountFactory::Braavos(inner) => {
                inner.sign_deployment_v3(deployment, query_only).await
            }
        }
    }
}
