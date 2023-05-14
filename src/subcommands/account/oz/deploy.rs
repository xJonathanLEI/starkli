use std::{io::Write, path::PathBuf, sync::Arc};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use starknet::{
    accounts::{AccountFactory, OpenZeppelinAccountFactory},
    core::{chain_id, types::FieldElement},
    providers::{
        jsonrpc::{HttpTransport, JsonRpcClient},
        SequencerGatewayProvider,
    },
    signers::{LocalWallet, SigningKey},
};

use crate::{
    account::{AccountConfig, AccountVariant, DeployedStatus, DeploymentStatus},
    utils::watch_tx,
    JsonRpcArgs,
};

#[derive(Debug, Parser)]
pub struct Deploy {
    #[clap(flatten)]
    jsonrpc: JsonRpcArgs,
    #[clap(long, help = "Path to keystore JSON file")]
    keystore: PathBuf,
    #[clap(help = "Path to the account config file")]
    file: PathBuf,
}

impl Deploy {
    pub async fn run(self) -> Result<()> {
        let jsonrpc_client = Arc::new(JsonRpcClient::new(HttpTransport::new(self.jsonrpc.rpc)));

        if !self.file.exists() {
            anyhow::bail!("account config file not found");
        }

        let mut account: AccountConfig =
            serde_json::from_reader(&mut std::fs::File::open(&self.file)?)?;

        #[allow(clippy::infallible_destructuring_match)]
        let oz_config = match &account.variant {
            AccountVariant::OpenZeppelin(inner) => inner,
        };

        let undeployed_status = match &account.deployment {
            DeploymentStatus::Undeployed(inner) => inner,
            DeploymentStatus::Deployed(_) => {
                anyhow::bail!("account already deployed");
            }
        };

        if !self.keystore.exists() {
            anyhow::bail!("keystore file not found");
        }

        let password = rpassword::prompt_password("Enter keystore password: ")?;
        let key = SigningKey::from_keystore(self.keystore, &password)?;

        // Makes sure we're using the right key
        if key.verifying_key().scalar() != oz_config.public_key {
            anyhow::bail!(
                "public key mismatch. Expected: {:#064x}; actual: {:#064x}.",
                oz_config.public_key,
                key.verifying_key().scalar()
            );
        }

        let chain_id = jsonrpc_client.chain_id().await?;

        let factory = OpenZeppelinAccountFactory::new(
            undeployed_status.class_hash,
            chain_id,
            LocalWallet::from_signing_key(key.clone()),
            jsonrpc_client.clone(),
        )
        .await?;

        let account_deployment = factory.deploy(undeployed_status.salt);

        let target_deployment_address = account.deploy_account_address()?;

        // Sanity check. We don't really need to check again here actually
        if account_deployment.address() != target_deployment_address {
            panic!("Unexpected account deployment address mismatch");
        }

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
                    let estimate_factory = OpenZeppelinAccountFactory::new(
                        undeployed_status.class_hash,
                        chain_id,
                        LocalWallet::from_signing_key(key),
                        sequencer_provider,
                    )
                    .await?;

                    estimate_factory
                        .deploy(undeployed_status.salt)
                        .estimate_fee()
                        .await?
                        .overall_fee
                }
                None => account_deployment.estimate_fee().await?.overall_fee,
            }
        };

        // TODO: make buffer configurable
        let estimated_fee_with_buffer = estimated_fee * 3 / 2;

        let estimated_fee: FieldElement = estimated_fee.into();
        let estimated_fee_with_buffer: FieldElement = estimated_fee_with_buffer.into();

        eprintln!(
            "The estimated account deployment fee is {}. \
            However, to avoid failure, fund at least:\n    {}",
            format!("{} ETH", estimated_fee.to_big_decimal(18)).bright_yellow(),
            format!("{} ETH", estimated_fee_with_buffer.to_big_decimal(18)).bright_yellow()
        );
        eprintln!(
            "to the following address:\n    {}",
            format!("{:#064x}", target_deployment_address).bright_yellow()
        );

        // TODO: add flag for skipping this manual confirmation step
        eprint!("Press [ENTER] once you've funded the address.");
        std::io::stdin().read_line(&mut String::new())?;

        // TODO: add option to check ETH balance before sending out tx
        let account_deployment_tx = account_deployment
            .max_fee(estimated_fee_with_buffer)
            .send()
            .await?
            .transaction_hash;
        eprintln!(
            "Account deployment transaction: {}",
            format!("{:#064x}", account_deployment_tx).bright_yellow()
        );

        // By default we wait for the tx to confirm so that we don't incorrectly mark the account
        // as deployed
        eprintln!(
            "Waiting for transaction {} to confirm. \
            If this process is interrupted, you will need to run `{}` to update the account file.",
            format!("{:#064x}", account_deployment_tx).bright_yellow(),
            "starkli account fetch".bright_yellow(),
        );
        watch_tx(&jsonrpc_client, account_deployment_tx).await?;

        account.deployment = DeploymentStatus::Deployed(DeployedStatus {
            class_hash: undeployed_status.class_hash,
            address: target_deployment_address,
        });

        // Never write directly to the original file to avoid data loss
        let mut temp_file_name = self
            .file
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("unable to determine file name"))?
            .to_owned();
        temp_file_name.push(".tmp");
        let mut temp_path = self.file.clone();
        temp_path.set_file_name(temp_file_name);

        let mut temp_file = std::fs::File::create(&temp_path)?;
        serde_json::to_writer_pretty(&mut temp_file, &account)?;
        temp_file.write_all(b"\n")?;
        std::fs::rename(temp_path, self.file)?;

        Ok(())
    }
}
