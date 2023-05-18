use std::{io::Write, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use starknet::{
    core::types::{BlockId, BlockTag, FieldElement, FunctionCall},
    macros::selector,
    providers::{
        jsonrpc::{HttpTransport, JsonRpcClient},
        Provider,
    },
};

use crate::{
    account::{
        AccountConfig, AccountVariant, AccountVariantType, DeployedStatus, DeploymentStatus,
        OzAccountConfig, KNOWN_ACCOUNT_CLASSES,
    },
    JsonRpcArgs,
};

#[derive(Debug, Parser)]
pub struct Fetch {
    #[clap(flatten)]
    jsonrpc: JsonRpcArgs,
    #[clap(long, help = "Overwrite the file if it already exists")]
    force: bool,
    #[clap(long, help = "Path to save the account config file")]
    output: PathBuf,
    #[clap(help = "Contract address")]
    address: String,
}

impl Fetch {
    pub async fn run(self) -> Result<()> {
        if self.output.exists() && !self.force {
            anyhow::bail!("account config file already exists");
        }

        let jsonrpc_client = JsonRpcClient::new(HttpTransport::new(self.jsonrpc.rpc));
        let address = FieldElement::from_hex_be(&self.address)?;

        let class_hash = jsonrpc_client
            .get_class_hash_at(BlockId::Tag(BlockTag::Pending), address)
            .await?;

        let known_class = match KNOWN_ACCOUNT_CLASSES
            .iter()
            .find(|class| class.class_hash == class_hash)
        {
            Some(class) => class,
            None => {
                eprintln!(
                    "{} is not a known account class hash. \
                    If you believe this is a bug, submit a PR to:",
                    format!("{:#064x}", class_hash).bright_yellow()
                );
                eprintln!("    https://github.com/xJonathanLEI/starkli");
                anyhow::bail!("unknown class hash: {:#064x}", class_hash);
            }
        };

        let account = match known_class.variant {
            AccountVariantType::OpenZeppelin => {
                let public_key = jsonrpc_client
                    .call(
                        FunctionCall {
                            contract_address: address,
                            entry_point_selector: selector!("getPublicKey"),
                            calldata: vec![],
                        },
                        BlockId::Tag(BlockTag::Pending),
                    )
                    .await?[0];

                AccountConfig {
                    version: 1,
                    variant: AccountVariant::OpenZeppelin(OzAccountConfig {
                        version: 1,
                        public_key,
                    }),
                    deployment: DeploymentStatus::Deployed(DeployedStatus {
                        class_hash,
                        address,
                    }),
                }
            }
        };

        let mut file = std::fs::File::create(&self.output)?;
        serde_json::to_writer_pretty(&mut file, &account)?;
        file.write_all(b"\n")?;

        eprintln!(
            "Downloaded new account config file: {}",
            std::fs::canonicalize(&self.output)?.display()
        );

        Ok(())
    }
}
