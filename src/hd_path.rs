use std::str::FromStr;

use clap::{builder::TypedValueParser, error::ErrorKind, Arg, Command, Error};
use coins_bip32::path::DerivationPath;
use colored::Colorize;
use sha2::{Digest, Sha256};

const EIP2645_LENGTH: usize = 6;

/// BIP-32 encoding of `2645'`
const EIP_2645_PURPOSE: u32 = 0x80000a55;

#[derive(Clone)]
pub struct DerivationPathParser;

#[derive(Clone)]
pub struct Eip2645PathParser;

/// An EIP-2645 HD path, required by the Starknet Ledger app. This type allows users to write hash-
/// based segments in text, instead of manually finding out the lowest 31 bits of the hash or such
/// texts.
///
/// Technically, the Ledger app only requires that the path:
///
/// - starts with `2645'`; and
/// - has 6 levels
///
/// In principle, Starkli should not enforce stricter path rules than those required by the app
/// itself, such as requiring hardening on all levels except `index`. However, it's always better
/// to start with stricter rules and relex them later as opposed to the other way around.
///
/// `eth_address_1` and `eth_address_2` do not make sense for Starknet. Users should just use them
/// as address ID.
///
/// Strictly speaking, the EIP-2645 standard only allows `layer` and `application` to be hashes.
/// However, since `eth_address_1` and `eth_address_2` are irrelevant for Starknet anyway, here we
/// allow those to be hashes too, enabling use cases like:
///
/// `m/2645'/starknet'/starkli'/dapp_xyz'/my_first_address'/0`
#[derive(Debug, Clone)]
pub struct Eip2645Path {
    layer: Eip2645Level,
    application: Eip2645Level,
    eth_address_1: Eip2645Level,
    eth_address_2: Eip2645Level,
    index: Eip2645Level,
}

#[derive(Debug, Clone)]
enum Eip2645Level {
    Hash(HashLevel),
    Raw(u32),
}

#[derive(Debug, Clone)]
struct HashLevel {
    text: String,
    hardened: bool,
}

impl TypedValueParser for DerivationPathParser {
    type Value = DerivationPath;

    fn parse_ref(
        &self,
        cmd: &Command,
        _arg: Option<&Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, Error> {
        if value.is_empty() {
            Err(cmd
                .clone()
                .error(ErrorKind::InvalidValue, "empty Ledger derivation path"))
        } else {
            match value.to_str() {
                Some(value) => match Eip2645Path::from_str(value) {
                    Ok(value) => Ok(value.into()),
                    Err(err) => Err(cmd.clone().error(
                        ErrorKind::InvalidValue,
                        format!(
                            "invalid Ledger derivation path: {}. Learn more about using \
                            Ledger with Starkli at https://book.starkli.rs/ledger",
                            err
                        ),
                    )),
                },
                None => Err(cmd.clone().error(
                    ErrorKind::InvalidValue,
                    "invalid Ledger derivation path: not UTF-8",
                )),
            }
        }
    }
}

impl TypedValueParser for Eip2645PathParser {
    type Value = Eip2645Path;

    fn parse_ref(
        &self,
        cmd: &Command,
        _arg: Option<&Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, Error> {
        if value.is_empty() {
            Err(cmd
                .clone()
                .error(ErrorKind::InvalidValue, "empty Ledger derivation path"))
        } else {
            match value.to_str() {
                Some(value) => match Eip2645Path::from_str(value) {
                    Ok(value) => Ok(value),
                    Err(err) => Err(cmd.clone().error(
                        ErrorKind::InvalidValue,
                        format!(
                            "invalid Ledger derivation path: {}. Learn more about using \
                            Ledger with Starkli at https://book.starkli.rs/ledger",
                            err
                        ),
                    )),
                },
                None => Err(cmd.clone().error(
                    ErrorKind::InvalidValue,
                    "invalid Ledger derivation path: not UTF-8",
                )),
            }
        }
    }
}
impl FromStr for Eip2645Path {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let segments: Vec<_> = s.split('/').collect();
        if segments.len() != EIP2645_LENGTH + 1 {
            anyhow::bail!("EIP-2645 paths must have {} levels", EIP2645_LENGTH);
        }
        if segments[0] != "m" {
            anyhow::bail!("HD wallet paths must start with \"m/\"");
        }

        // Here we allow the first level to be empty so that users can do something like:
        //
        // `m//starknet'/starkli'/0'/0'/0`
        //
        // to avoid writing `2645'` over and over again.
        if !segments[1].is_empty() {
            let prefix: Eip2645Level = segments[1].parse()?;
            if Into::<u32>::into(&prefix) != EIP_2645_PURPOSE {
                anyhow::bail!("EIP-2645 paths must start with \"m/2645'/\"");
            }
        }

        let path = Self {
            layer: segments[2].parse()?,
            application: segments[3].parse()?,
            eth_address_1: segments[4].parse()?,
            eth_address_2: segments[5].parse()?,
            index: segments[6].parse()?,
        };

        // These are not enforced by Ledger (for now) but are nice to have security properties
        if !path.layer.is_hardened() {
            anyhow::bail!("the \"layer\" level of an EIP-2645 path must be hardened");
        }
        if !path.application.is_hardened() {
            anyhow::bail!("the \"application\" level of an EIP-2645 path must be hardened");
        }
        if !path.eth_address_1.is_hardened() {
            anyhow::bail!("the \"eth_address_1\" level of an EIP-2645 path must be hardened");
        }
        if !path.eth_address_2.is_hardened() {
            anyhow::bail!("the \"eth_address_2\" level of an EIP-2645 path must be hardened");
        }

        // In the future, certain wallets might utilize sequential `index` values for key discovery,
        // so it might be a good idea for us to disallow using hash-based values for `index` here.
        if matches!(path.index, Eip2645Level::Hash(_)) {
            anyhow::bail!("the \"index\" level must be a number");
        }

        // These are allowed but we should serve a warning
        // TODO: add environment variable to surpress warning
        match &path.eth_address_1 {
            Eip2645Level::Hash(_) => {
                eprintln!(
                    "{}",
                    "WARNING: using a non-numerical values for \"eth_address_1\" might make \
                    automatic key discovery difficult or impossible. Learn more at \
                    https://book.starkli.rs/ledger"
                        .bright_magenta()
                );
            }
            Eip2645Level::Raw(raw) => {
                if (*raw) & 0x7fffffff != 0 {
                    eprintln!(
                        "{}",
                        "WARNING: using any value other than `0'` for \"eth_address_1\" might \
                        make automatic key discovery difficult or impossible. Learn more at \
                        https://book.starkli.rs/ledger"
                            .bright_magenta()
                    );
                }
            }
        }
        match &path.eth_address_2 {
            Eip2645Level::Hash(_) => {
                eprintln!(
                    "{}",
                    "WARNING: using a non-numerical values for \"eth_address_2\" might make \
                    automatic key discovery difficult or impossible. Learn more at \
                    https://book.starkli.rs/ledger"
                        .bright_magenta()
                );
            }
            Eip2645Level::Raw(raw) => {
                if (*raw) & 0x7fffffff > 100 {
                    eprintln!(
                        "{}",
                        "WARNING: using a large value for \"eth_address_2\" might \
                        make automatic key discovery difficult. Learn more at \
                        https://book.starkli.rs/ledger"
                            .bright_magenta()
                    );
                }
            }
        }
        if path.index.is_hardened() {
            eprintln!(
                "{}",
                "WARNING: hardening \"index\" is non-standard and it might \
                make automatic key discovery difficult or impossible. Learn more at \
                https://book.starkli.rs/ledger"
                    .bright_magenta()
            );
        }
        if u32::from(&path.index) & 0x7fffffff > 100 {
            eprintln!(
                "{}",
                "WARNING: using a large value for \"index\" might \
                make automatic key discovery difficult. Learn more at \
                https://book.starkli.rs/ledger"
                    .bright_magenta()
            );
        }

        Ok(path)
    }
}

impl Eip2645Level {
    fn is_hardened(&self) -> bool {
        match self {
            Self::Hash(hash) => hash.hardened,
            Self::Raw(raw) => raw & 0x80000000 > 0,
        }
    }
}

impl std::fmt::Display for Eip2645Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "m/2645'/{}/{}/{}/{}/{}",
            self.layer, self.application, self.eth_address_1, self.eth_address_2, self.index
        )
    }
}

impl std::fmt::Display for Eip2645Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_hardened() {
            write!(f, "{}'", u32::from(self) & 0x7fffffff)
        } else {
            write!(f, "{}", u32::from(self))
        }
    }
}

impl FromStr for Eip2645Level {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim() != s || s.split_whitespace().count() != 1 {
            anyhow::bail!("path must not contain whitespaces");
        }

        let (body, harden_notation) = if s.ends_with('\'') {
            (&s[0..(s.len() - 1)], true)
        } else {
            (s, false)
        };

        if body.chars().all(|char| char.is_ascii_digit()) {
            // It's interpreted as number even if we end up failing to parse
            let raw_node = body
                .parse::<u32>()
                .map_err(|err| anyhow::anyhow!("invalid path level \"{}\": {}", body, err))?;

            if harden_notation {
                if raw_node & 0x80000000 > 0 {
                    anyhow::bail!("`'` appended to an already-hardened value of {}", raw_node);
                }

                Ok(Self::Raw(raw_node | 0x80000000))
            } else {
                Ok(Self::Raw(raw_node))
            }
        } else {
            Ok(Self::Hash(HashLevel {
                text: body.to_owned(),
                hardened: harden_notation,
            }))
        }
    }
}

impl From<Eip2645Path> for DerivationPath {
    fn from(value: Eip2645Path) -> Self {
        vec![
            EIP_2645_PURPOSE,
            (&value.layer).into(),
            (&value.application).into(),
            (&value.eth_address_1).into(),
            (&value.eth_address_2).into(),
            (&value.index).into(),
        ]
        .into()
    }
}

impl From<&Eip2645Level> for u32 {
    fn from(value: &Eip2645Level) -> Self {
        match value {
            Eip2645Level::Hash(level) => {
                let mut hasher = Sha256::new();
                hasher.update(level.text.as_bytes());
                let hash = hasher.finalize();

                // Safe to unwrap as SHA256 output is fixed size
                let node = u32::from_be_bytes(hash.as_slice()[28..].try_into().unwrap());

                // We assume text-based nodes are always hardened
                if level.hardened {
                    node | 0x80000000
                } else {
                    node & 0x7fffffff
                }
            }
            Eip2645Level::Raw(raw) => *raw,
        }
    }
}
