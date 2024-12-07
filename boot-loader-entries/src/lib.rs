use core::fmt;
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

mod parser;

/// A menu entry key, containing a fragment of configuration for the boot loader.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EntryKey {
    Linux(PathBuf),
    Devicetree(PathBuf),
    Options(Vec<String>),
}

impl EntryKey {
    pub fn file<'a>(&'a self) -> Option<&'a Path> {
        match self {
            EntryKey::Linux(path) => Some(path),
            EntryKey::Devicetree(path) => Some(path),
            EntryKey::Options(_) => None,
        }
    }
}

impl fmt::Display for EntryKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryKey::Linux(path) => write!(f, "linux {}", path.display()),
            EntryKey::Devicetree(path) => write!(f, "devicetree {}", path.display()),
            EntryKey::Options(options) => write!(f, "options {}", options.join(" ")),
        }
    }
}

#[derive(Clone, thiserror::Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("erroneous boot entry")]
    ErroneousEntry(String),
}

impl From<nom::Err<nom::error::Error<&str>>> for Error {
    fn from(value: nom::Err<nom::error::Error<&str>>) -> Self {
        match value {
            nom::Err::Incomplete(_) => panic!("Parser reported incomplete input. This is a bug"),
            nom::Err::Error(error) => Error::ErroneousEntry(error.input.to_string()),
            nom::Err::Failure(error) => Error::ErroneousEntry(error.input.to_string()),
        }
    }
}

impl FromStr for EntryKey {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let (rest, entry) = parser::entry_key(input).map_err(Into::<Self::Err>::into)?;
        match rest {
            "" => Ok(entry),
            _ => Err(Error::ErroneousEntry(format!(
                "trailing garbage: \"{}\"",
                rest
            ))),
        }
    }
}

/// A boot loader entry for this system. This datum represents a "Type #1" (text-based) boot entry
/// according to the [UAPI group Boot Loader Specification][https://uapi-group.org/specifications/specs/boot_loader_specification/]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BootEntry {
    pub keys: Vec<EntryKey>,
}

impl fmt::Display for BootEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for key in &self.keys {
            key.fmt(f)?;
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl FromStr for BootEntry {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let (rest, entry) = parser::boot_entry(input).map_err(Into::<Self::Err>::into)?;
        match rest {
            "" => Ok(entry),
            _ => Err(Error::ErroneousEntry(format!(
                "trailing garbage: \"{}\"",
                rest
            ))),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn boot_entry_with_trailing_junk() {
        let result = BootEntry::from_str("linux /Image\ndevisetree");
        assert!(result.is_err());
    }
}
