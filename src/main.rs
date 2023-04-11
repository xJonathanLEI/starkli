use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use url::Url;

use crate::subcommands::*;

mod subcommands;
mod utils;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Subcommands,
}

#[derive(Debug, Clone, Parser)]
struct JsonRpcArgs {
    #[clap(
        long = "rpc",
        env = "STARKNET_RPC",
        help = "Starknet JSON-RPC endpoint"
    )]
    rpc: Url,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
    //
    // Local utilities
    //
    #[clap(about = "Calculate selector from name")]
    Selector(Selector),
    #[clap(about = "Calculate class hash from any contract artifacts (Sierra, casm, legacy)")]
    ClassHash(ClassHash),
    #[clap(about = "Encode string into felt with the Cairo short string representation")]
    ToCairoString(ToCairoString),
    #[clap(about = "Decode string from felt with the Cairo short string representation")]
    ParseCairoString(ParseCairoString),
    #[clap(about = "Prints the montgomery representation of a field element")]
    Mont(Mont),
    //
    // JSON-RPC query client
    //
    #[clap(about = "Get Starknet transaction by hash")]
    Transaction(Transaction),
    #[clap(about = "Get latest block number")]
    BlockNumber(BlockNumber),
    #[clap(about = "Get Starknet block")]
    Block(Block),
    #[clap(about = "Get Starknet block timestamp only")]
    BlockTime(BlockTime),
    #[clap(about = "Get transaction receipt by hash")]
    TransactionReceipt(TransactionReceipt),
    #[clap(about = "Get Starknet network ID")]
    ChainId(ChainId),
    //
    // Misc
    //
    #[clap(about = "Generate shell completions script")]
    Completions(Completions),
}

#[tokio::main]
async fn main() {
    if let Err(err) = run_command(Cli::parse()).await {
        eprintln!("{}", format!("Error: {err}").red());
        std::process::exit(1);
    }
}

async fn run_command(cli: Cli) -> Result<()> {
    match cli.command {
        Subcommands::Selector(cmd) => cmd.run(),
        Subcommands::ClassHash(cmd) => cmd.run(),
        Subcommands::ToCairoString(cmd) => cmd.run(),
        Subcommands::ParseCairoString(cmd) => cmd.run(),
        Subcommands::Mont(cmd) => cmd.run(),
        Subcommands::Transaction(cmd) => cmd.run().await,
        Subcommands::BlockNumber(cmd) => cmd.run().await,
        Subcommands::Block(cmd) => cmd.run().await,
        Subcommands::BlockTime(cmd) => cmd.run().await,
        Subcommands::TransactionReceipt(cmd) => cmd.run().await,
        Subcommands::ChainId(cmd) => cmd.run().await,
        Subcommands::Completions(cmd) => cmd.run(),
    }
}
