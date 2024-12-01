use async_trait::async_trait;
use starknet::{
    accounts::{
        AccountFactory, PreparedAccountDeploymentV1, PreparedAccountDeploymentV3,
        RawAccountDeploymentV1, RawAccountDeploymentV3,
    },
    core::types::{BlockId, BlockTag, Felt},
    providers::Provider,
    signers::{Signer, SignerInteractivityContext},
};
use starknet_crypto::poseidon_hash_many;

pub struct BraavosAccountFactory<S, P> {
    class_hash: Felt,
    base_class_hash: Felt,
    chain_id: Felt,
    signer_public_key: Felt,
    signer: S,
    provider: P,
    block_id: BlockId,
}

impl<S, P> BraavosAccountFactory<S, P>
where
    S: Signer,
{
    pub async fn new(
        class_hash: Felt,
        base_class_hash: Felt,
        chain_id: Felt,
        signer: S,
        provider: P,
    ) -> Result<Self, S::GetPublicKeyError> {
        let signer_public_key = signer.get_public_key().await?;
        Ok(Self {
            class_hash,
            base_class_hash,
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

    #[allow(clippy::misnamed_getters)]
    fn class_hash(&self) -> Felt {
        self.base_class_hash
    }

    fn calldata(&self) -> Vec<Felt> {
        vec![self.signer_public_key]
    }

    fn chain_id(&self) -> Felt {
        self.chain_id
    }

    fn provider(&self) -> &Self::Provider {
        &self.provider
    }

    fn is_signer_interactive(&self) -> bool {
        self.signer
            .is_interactive(SignerInteractivityContext::Other)
    }

    fn block_id(&self) -> BlockId {
        self.block_id
    }

    async fn sign_deployment_v1(
        &self,
        deployment: &RawAccountDeploymentV1,
        query_only: bool,
    ) -> Result<Vec<Felt>, Self::SignError> {
        let tx_hash = PreparedAccountDeploymentV1::from_raw(deployment.clone(), self)
            .transaction_hash(query_only);

        let signature = self.signer.sign_hash(&tx_hash).await?;

        let mut aux_data = vec![
            // account_implementation
            self.class_hash,
            // signer_type
            Felt::ZERO,
            // secp256r1_signer.x.low
            Felt::ZERO,
            // secp256r1_signer.x.high
            Felt::ZERO,
            // secp256r1_signer.y.low
            Felt::ZERO,
            // secp256r1_signer.y.high
            Felt::ZERO,
            // multisig_threshold
            Felt::ZERO,
            // withdrawal_limit_low
            Felt::ZERO,
            // fee_rate
            Felt::ZERO,
            // stark_fee_rate
            Felt::ZERO,
            // chain_id
            self.chain_id,
        ];

        let aux_hash = poseidon_hash_many(&aux_data);

        let aux_signature = self.signer.sign_hash(&aux_hash).await?;

        let mut full_signature_payload = vec![signature.r, signature.s];
        full_signature_payload.append(&mut aux_data);
        full_signature_payload.push(aux_signature.r);
        full_signature_payload.push(aux_signature.s);

        Ok(full_signature_payload)
    }

    async fn sign_deployment_v3(
        &self,
        deployment: &RawAccountDeploymentV3,
        query_only: bool,
    ) -> Result<Vec<Felt>, Self::SignError> {
        let tx_hash = PreparedAccountDeploymentV3::from_raw(deployment.clone(), self)
            .transaction_hash(query_only);

        let signature = self.signer.sign_hash(&tx_hash).await?;

        let mut aux_data = vec![
            // account_implementation
            self.class_hash,
            // signer_type
            Felt::ZERO,
            // secp256r1_signer.x.low
            Felt::ZERO,
            // secp256r1_signer.x.high
            Felt::ZERO,
            // secp256r1_signer.y.low
            Felt::ZERO,
            // secp256r1_signer.y.high
            Felt::ZERO,
            // multisig_threshold
            Felt::ZERO,
            // withdrawal_limit_low
            Felt::ZERO,
            // fee_rate
            Felt::ZERO,
            // stark_fee_rate
            Felt::ZERO,
            // chain_id
            self.chain_id,
        ];

        let aux_hash = poseidon_hash_many(&aux_data);

        let aux_signature = self.signer.sign_hash(&aux_hash).await?;

        let mut full_signature_payload = vec![signature.r, signature.s];
        full_signature_payload.append(&mut aux_data);
        full_signature_payload.push(aux_signature.r);
        full_signature_payload.push(aux_signature.s);

        Ok(full_signature_payload)
    }
}
