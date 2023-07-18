use std::path::PathBuf;

use clap::{builder::TypedValueParser, error::ErrorKind, Arg, Command, Error};

#[derive(Clone)]
pub struct ExpandedPathbufParser;

impl TypedValueParser for ExpandedPathbufParser {
    type Value = PathBuf;

    fn parse_ref(
        &self,
        cmd: &Command,
        _arg: Option<&Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, Error> {
        if value.is_empty() {
            Err(cmd.clone().error(ErrorKind::InvalidValue, "empty path"))
        } else {
            let path = match value.to_str() {
                Some(value) => PathBuf::from(shellexpand::tilde(value).into_owned()),
                None => PathBuf::from(value),
            };

            Ok(path)
        }
    }
}
