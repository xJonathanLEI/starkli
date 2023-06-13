use anyhow::Result;
use async_trait::async_trait;
use starknet::{core::types::FieldElement, providers::Provider};

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait ChainIdSource {
    async fn get_chain_id(&self) -> Result<FieldElement>;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<T> ChainIdSource for T
where
    T: Provider + Send + Sync,
{
    async fn get_chain_id(&self) -> Result<FieldElement> {
        self.chain_id()
            .await
            .map_err(|err| anyhow::anyhow!("unable to get chain id: {err}"))
    }
}
