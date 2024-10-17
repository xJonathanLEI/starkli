use anyhow::Result;
use clap::{CommandFactory, Parser};
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

async fn run_command(cli: Cli) -> Result<()> {
    match (cli.version, cli.command) {
        (false, None) => Ok(Cli::command().print_help()?),
        (true, _) => {
            println!(
                "{}",
                if cli.verbose {
                    VERSION_STRING_VERBOSE
                } else {
                    VERSION_STRING
                }
            );

            Ok(())
        }
        (false, Some(command)) => match command {
            Subcommands::Selector(cmd) => cmd.run(),
            Subcommands::ClassHash(cmd) => cmd.run(),
            Subcommands::Abi(cmd) => cmd.run(),
            Subcommands::ToCairoString(cmd) => cmd.run(),
            Subcommands::ParseCairoString(cmd) => cmd.run(),
            Subcommands::Mont(cmd) => cmd.run(),
            Subcommands::Call(cmd) => cmd.run().await,
            Subcommands::Transaction(cmd) => cmd.run().await,
            Subcommands::BlockNumber(cmd) => cmd.run().await,
            Subcommands::BlockHash(cmd) => cmd.run().await,
            Subcommands::Block(cmd) => cmd.run().await,
            Subcommands::BlockTime(cmd) => cmd.run().await,
            Subcommands::StateUpdate(cmd) => cmd.run().await,
            Subcommands::BlockTraces(cmd) => cmd.run().await,
            Subcommands::Status(cmd) => cmd.run().await,
            Subcommands::Receipt(cmd) => cmd.run().await,
            Subcommands::Trace(cmd) => cmd.run().await,
            Subcommands::ChainId(cmd) => cmd.run().await,
            Subcommands::Balance(cmd) => cmd.run().await,
            Subcommands::Nonce(cmd) => cmd.run().await,
            Subcommands::Storage(cmd) => cmd.run().await,
            Subcommands::ClassHashAt(cmd) => cmd.run().await,
            Subcommands::ClassByHash(cmd) => cmd.run().await,
            Subcommands::ClassAt(cmd) => cmd.run().await,
            Subcommands::Syncing(cmd) => cmd.run().await,
            Subcommands::SpecVersion(cmd) => cmd.run().await,
            Subcommands::Signer(cmd) => cmd.run().await,
            Subcommands::Ledger(cmd) => cmd.run().await,
            Subcommands::Eip2645(cmd) => cmd.run(),
            Subcommands::Account(cmd) => cmd.run().await,
            Subcommands::Invoke(cmd) => cmd.run().await,
            Subcommands::Declare(cmd) => cmd.run().await,
            Subcommands::Deploy(cmd) => cmd.run().await,
            Subcommands::Completions(cmd) => cmd.run(),
            Subcommands::Lab(cmd) => cmd.run(),
        },
    }
}
