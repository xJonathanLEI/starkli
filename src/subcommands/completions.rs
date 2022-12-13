use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};

use crate::Cli;

#[derive(Debug, Parser)]
pub struct Completions {
    #[clap(help = "Shell name")]
    shell: Shell,
}

impl Completions {
    pub fn run(self) -> Result<()> {
        generate(
            self.shell,
            &mut Cli::command(),
            env!("CARGO_PKG_NAME"),
            &mut std::io::stdout(),
        );

        Ok(())
    }
}
