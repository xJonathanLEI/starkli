use async_trait::async_trait;
use starknet::{
    accounts::{AccountFactory, PreparedAccountDeployment, RawAccountDeployment},
    core::{
        crypto::compute_hash_on_elements,
        types::{BlockId, BlockTag, FieldElement},
    },
    macros::selector,
    providers::Provider,
    signers::Signer,
};

pub struct BraavosAccountFactory<S, P> {
    proxy_class_hash: FieldElement,
    mock_impl_class_hash: FieldElement,
    impl_class_hash: FieldElement,
    chain_id: FieldElement,
    signer_public_key: FieldElement,
    signer: S,
    provider: P,
    block_id: BlockId,
}

impl<S, P> BraavosAccountFactory<S, P>
where
    S: Signer,
{
    pub async fn new(
        proxy_class_hash: FieldElement,
        mock_impl_class_hash: FieldElement,
        impl_class_hash: FieldElement,
        chain_id: FieldElement,
        signer: S,
        provider: P,
    ) -> Result<Self, S::GetPublicKeyError> {
        let signer_public_key = signer.get_public_key().await?;
        Ok(Self {
            proxy_class_hash,
            mock_impl_class_hash,
            impl_class_hash,
            chain_id,
            signer_public_key: signer_public_key.scalar(),
            signer,
            provider,
            block_id: BlockId::Tag(BlockTag::Latest),
        })
    }

    pub fn set_block_id(&mut self, block_id: BlockId) -> &Self {
        self.block_id = block_id;
        self
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<S, P> AccountFactory for BraavosAccountFactory<S, P>
where
    S: Signer + Sync + Send,
    P: Provider + Sync + Send,
{
    type Provider = P;
    type SignError = S::SignError;

    fn class_hash(&self) -> FieldElement {
        self.proxy_class_hash
    }

    fn calldata(&self) -> Vec<FieldElement> {
        vec![
            self.mock_impl_class_hash,
            selector!("initializer"),
            FieldElement::ONE,
            self.signer_public_key,
        ]
    }

    fn chain_id(&self) -> FieldElement {
        self.chain_id
    }

    fn provider(&self) -> &Self::Provider {
        &self.provider
    }

    fn block_id(&self) -> BlockId {
        self.block_id
    }

    async fn sign_deployment(
        &self,
        deployment: &RawAccountDeployment,
    ) -> Result<Vec<FieldElement>, Self::SignError> {
        let tx_hash =
            PreparedAccountDeployment::from_raw(deployment.clone(), self).transaction_hash();

        let sig_hash = compute_hash_on_elements(&[
            tx_hash,
            self.impl_class_hash,
            FieldElement::ZERO,
            FieldElement::ZERO,
            FieldElement::ZERO,
            FieldElement::ZERO,
            FieldElement::ZERO,
            FieldElement::ZERO,
            FieldElement::ZERO,
        ]);

        let signature = self.signer.sign_hash(&sig_hash).await?;

        Ok(vec![
            signature.r,
            signature.s,
            self.impl_class_hash,
            FieldElement::ZERO,
            FieldElement::ZERO,
            FieldElement::ZERO,
            FieldElement::ZERO,
            FieldElement::ZERO,
            FieldElement::ZERO,
            FieldElement::ZERO,
        ])
    }
}
