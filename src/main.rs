use anyhow::Result;
use clap::{Parser, Subcommand};
use url::Url;

use crate::subcommands::{
    block_number::BlockNumber, class_hash::ClassHash, completions::Completions,
    get_block::GetBlock, get_transaction::GetTransaction, selector::Selector,
};

mod subcommands;

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
        help = "StarkNet JSON-RPC endpoint"
    )]
    rpc: Url,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
    #[clap(about = "Calculate selector from name")]
    Selector(Selector),
    #[clap(about = "Calculate class hash from compiled contract artifact")]
    ClassHash(ClassHash),
    #[clap(about = "Get StarkNet transaction by hash")]
    GetTransaction(GetTransaction),
    #[clap(about = "Get latest block number")]
    BlockNumber(BlockNumber),
    #[clap(about = "Get StarkNet block")]
    GetBlock(GetBlock),
    #[clap(about = "Generate shell completions script")]
    Completions(Completions),
}

#[tokio::main]
async fn main() {
    if let Err(err) = run_command(Cli::parse()).await {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}

async fn run_command(cli: Cli) -> Result<()> {
    match cli.command {
        Subcommands::Selector(cmd) => cmd.run(),
        Subcommands::ClassHash(cmd) => cmd.run(),
        Subcommands::GetTransaction(cmd) => cmd.run().await,
        Subcommands::BlockNumber(cmd) => cmd.run().await,
        Subcommands::GetBlock(cmd) => cmd.run().await,
        Subcommands::Completions(cmd) => cmd.run(),
    }
}
