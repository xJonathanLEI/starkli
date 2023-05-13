use anyhow::Result;
use clap::Parser;
use starknet::signers::SigningKey;

#[derive(Debug, Parser)]
pub struct GenKeypair {}

impl GenKeypair {
    pub fn run(self) -> Result<()> {
        let key = SigningKey::from_random();

        println!("Private key : {:#064x}", key.secret_scalar());
        println!("Public key  : {:#064x}", key.verifying_key().scalar());

        Ok(())
    }
}
