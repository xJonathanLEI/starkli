use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use serde_json_pythonic::to_string_pythonic;
use starknet::core::types::contract::{legacy::LegacyContractClass, CompiledClass, SierraClass};

use crate::{path::ExpandedPathbufParser, utils::print_colored_json};

#[derive(Debug, Parser)]
pub struct Abi {
    #[clap(
        long,
        help = "Present the ABI as a flattened string in a Pythoic style"
    )]
    flatten: bool,
    #[clap(
        long,
        help = "When --flatten is used, serialize the ABI in the Pythoic style instead of compact"
    )]
    pythonic: bool,
    #[clap(
        value_parser = ExpandedPathbufParser,
        help = "Path to contract artifact file"
    )]
    file: PathBuf,
}

impl Abi {
    pub fn run(self) -> Result<()> {
        if self.pythonic && !self.flatten {
            anyhow::bail!("--pythonic can only be used with --flatten");
        }

        // Working around a deserialization bug in `starknet-rs`:
        //   https://github.com/xJonathanLEI/starknet-rs/issues/392

        if let Ok(class) =
            serde_json::from_reader::<_, SierraClass>(std::fs::File::open(&self.file)?)
        {
            let abi = class.abi;
            if self.flatten {
                if self.pythonic {
                    println!("{}", to_string_pythonic(&abi)?);
                } else {
                    println!("{}", serde_json::to_string(&abi)?);
                }
            } else {
                print_colored_json(&abi)?;
            }
        } else if let Ok(class) =
            serde_json::from_reader::<_, LegacyContractClass>(std::fs::File::open(&self.file)?)
        {
            let abi = class.abi;
            if self.flatten {
                if self.pythonic {
                    println!("{}", to_string_pythonic(&abi)?);
                } else {
                    println!("{}", serde_json::to_string(&abi)?);
                }
            } else {
                print_colored_json(&abi)?;
            }
        } else if serde_json::from_reader::<_, CompiledClass>(std::fs::File::open(self.file)?)
            .is_ok()
        {
            anyhow::bail!("cannot extract ABI from casm");
        } else {
            anyhow::bail!("failed to parse contract artifact");
        }

        Ok(())
    }
}
