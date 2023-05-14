use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use starknet::{
    accounts::SingleOwnerAccount,
    contract::ContractFactory,
    core::{chain_id, types::FieldElement},
    providers::{
        jsonrpc::{HttpTransport, JsonRpcClient},
        SequencerGatewayProvider,
    },
    signers::{LocalWallet, SigningKey},
};

use crate::{
    account::{AccountConfig, DeploymentStatus},
    utils::watch_tx,
    JsonRpcArgs,
};

#[derive(Debug, Parser)]
pub struct Deploy {
    #[clap(flatten)]
    jsonrpc: JsonRpcArgs,
    #[clap(long, help = "Do not derive contract address from deployer address")]
    not_unique: bool,
    #[clap(long, help = "Path to keystore JSON file")]
    keystore: PathBuf,
    #[clap(long, help = "Path to account config JSON file")]
    account: PathBuf,
    #[clap(long, help = "Wait for the transaction to confirm")]
    watch: bool,
    #[clap(help = "Class hash")]
    class_hash: String,
    #[clap(help = "Raw constructor arguments")]
    ctor_args: Vec<String>,
}

impl Deploy {
    pub async fn run(self) -> Result<()> {
        let jsonrpc_client = Arc::new(JsonRpcClient::new(HttpTransport::new(self.jsonrpc.rpc)));

        if !self.keystore.exists() {
            anyhow::bail!("keystore file not found");
        }

        if !self.account.exists() {
            anyhow::bail!("account config file not found");
        }

        let class_hash = FieldElement::from_hex_be(&self.class_hash)?;
        let ctor_args = self
            .ctor_args
            .iter()
            .map(|item| item.parse())
            .collect::<Result<Vec<FieldElement>, _>>()?;

        // TODO: add option for manually setting salt
        let salt = SigningKey::from_random().secret_scalar();

        // TODO: refactor account & wallet loading

        let account_config: AccountConfig =
            serde_json::from_reader(&mut std::fs::File::open(&self.account)?)?;

        let account_address = match account_config.deployment {
            DeploymentStatus::Undeployed(_) => anyhow::bail!("account not deployed"),
            DeploymentStatus::Deployed(inner) => inner.address,
        };

        let password = rpassword::prompt_password("Enter keystore password: ")?;
        let key = SigningKey::from_keystore(self.keystore, &password)?;

        let chain_id = jsonrpc_client.chain_id().await?;

        let account = SingleOwnerAccount::new(
            jsonrpc_client.clone(),
            LocalWallet::from_signing_key(key.clone()),
            account_address,
            chain_id,
        );

        // TODO: allow custom UDC
        let factory = ContractFactory::new(class_hash, account);

        // TODO: pre-compute and show target deployment address

        let contract_deployment = factory.deploy(&ctor_args, salt, !self.not_unique);

        // TODO: add option for manually specifying fees
        let estimated_fee = {
            // Extremely hacky workaround for a `pathfinder` bug:
            //   https://github.com/eqlabs/pathfinder/issues/1082

            let sequencer_fallback = if chain_id == chain_id::MAINNET {
                Some(SequencerGatewayProvider::starknet_alpha_mainnet())
            } else if chain_id == chain_id::TESTNET {
                Some(SequencerGatewayProvider::starknet_alpha_goerli())
            } else if chain_id == chain_id::TESTNET2 {
                Some(SequencerGatewayProvider::starknet_alpha_goerli_2())
            } else {
                None
            };

            match sequencer_fallback {
                Some(sequencer_provider) => {
                    let estimate_account = SingleOwnerAccount::new(
                        sequencer_provider,
                        LocalWallet::from_signing_key(key),
                        account_address,
                        chain_id,
                    );

                    let estimate_factory = ContractFactory::new(class_hash, estimate_account);

                    estimate_factory
                        .deploy(&ctor_args, salt, !self.not_unique)
                        .estimate_fee()
                        .await?
                        .overall_fee
                }
                None => contract_deployment.estimate_fee().await?.overall_fee,
            }
        };

        // TODO: make buffer configurable
        let estimated_fee_with_buffer = estimated_fee * 3 / 2;

        eprintln!(
            "Deploying class {} with salt {}...",
            format!("{:#064x}", class_hash).bright_yellow(),
            format!("{:#064x}", salt).bright_yellow()
        );

        let deployment_tx = contract_deployment
            .max_fee(estimated_fee_with_buffer.into())
            .send()
            .await?
            .transaction_hash;
        eprintln!(
            "Contract deployment transaction: {}",
            format!("{:#064x}", deployment_tx).bright_yellow()
        );

        if self.watch {
            eprintln!(
                "Waiting for transaction {} to confirm...",
                format!("{:#064x}", deployment_tx).bright_yellow(),
            );
            watch_tx(&jsonrpc_client, deployment_tx).await?;
        }

        Ok(())
    }
}
