use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use starknet::core::types::{
    contract::{legacy::LegacyContractClass, CompiledClass, SierraClass},
    CompressedLegacyContractClass, FlattenedSierraClass,
};

use crate::{path::ExpandedPathbufParser, utils::parse_compressed_legacy_class};

#[derive(Debug, Parser)]
pub struct ClassHash {
    #[clap(
        value_parser = ExpandedPathbufParser,
        help = "Path to contract artifact file"
    )]
    file: PathBuf,
}

impl ClassHash {
    pub fn run(self) -> Result<()> {
        // Working around a deserialization bug in `starknet-rs`:
        //   https://github.com/xJonathanLEI/starknet-rs/issues/392

        let class_hash = if let Ok(class) =
            serde_json::from_reader::<_, SierraClass>(std::fs::File::open(&self.file)?)
        {
            class.class_hash()?
        } else if let Ok(class) =
            serde_json::from_reader::<_, CompiledClass>(std::fs::File::open(&self.file)?)
        {
            class.class_hash()?
        } else if let Ok(class) =
            serde_json::from_reader::<_, LegacyContractClass>(std::fs::File::open(&self.file)?)
        {
            class.class_hash()?
        } else if let Ok(class) =
            serde_json::from_reader::<_, FlattenedSierraClass>(std::fs::File::open(&self.file)?)
        {
            class.class_hash()
        } else if let Ok(class) = serde_json::from_reader::<_, CompressedLegacyContractClass>(
            std::fs::File::open(self.file)?,
        ) {
            parse_compressed_legacy_class(class)?.class_hash()?
        } else {
            anyhow::bail!("failed to parse contract artifact");
        };

        println!("{class_hash:#064x}");

        Ok(())
    }
}
