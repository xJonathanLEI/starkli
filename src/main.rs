use clap::{Parser, Subcommand};

use crate::subcommands::selector::Selector;

mod subcommands;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Subcommands,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
    #[clap(about = "Calculate selector from name")]
    Selector(Selector),
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Subcommands::Selector(cmd) => {
            cmd.run();
        }
    }
}
