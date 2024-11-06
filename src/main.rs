use clap::Parser;
use colored::Colorize;

use crate::utils::*;

mod account;
mod account_factory;
mod address_book;
mod block_id;
mod casm;
mod chain_id;
mod compiler;
mod decode;
mod error;
mod fee;
mod hd_path;
mod network;
mod path;
mod profile;
mod provider;
mod signer;
mod subcommands;
mod utils;
mod verbosity;

#[tokio::main]
async fn main() {
    if let Err(err) = run_command(Cli::parse()).await {
        eprintln!("{}", format!("Error: {err}").red());
        std::process::exit(1);
    }
}
