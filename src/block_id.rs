use clap::{builder::TypedValueParser, error::ErrorKind, Arg, Command, Error};
use regex::Regex;
use starknet::core::types::{BlockId, BlockTag, Felt};

#[derive(Clone)]
pub struct BlockIdParser;

impl TypedValueParser for BlockIdParser {
    type Value = BlockId;

    fn parse_ref(
        &self,
        cmd: &Command,
        _arg: Option<&Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, Error> {
        if value.is_empty() {
            Err(cmd.clone().error(ErrorKind::InvalidValue, "empty block ID"))
        } else {
            match value.to_str() {
                Some(value) => {
                    let regex_block_number = Regex::new("^[0-9]{1,}$").unwrap();

                    if value == "latest" {
                        Ok(BlockId::Tag(BlockTag::Latest))
                    } else if value == "pending" {
                        Ok(BlockId::Tag(BlockTag::Pending))
                    } else if regex_block_number.is_match(value) {
                        Ok(BlockId::Number(value.parse::<u64>().map_err(|err| {
                            cmd.clone().error(
                                ErrorKind::InvalidValue,
                                format!("invalid block number: {}", err),
                            )
                        })?))
                    } else {
                        Ok(BlockId::Hash(Felt::from_hex(value).map_err(|err| {
                            cmd.clone().error(
                                ErrorKind::InvalidValue,
                                format!("invalid block hash: {}", err),
                            )
                        })?))
                    }
                }
                None => Err(cmd
                    .clone()
                    .error(ErrorKind::InvalidValue, "invalid block ID")),
            }
        }
    }
}
