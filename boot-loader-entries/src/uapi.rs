use core::fmt;
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::parser;

/// A menu entry key, containing a fragment of configuration for the boot loader.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize, ::serde::Serialize))]
pub enum EntryKey {
    Title(String),
    Linux(PathBuf),
    Devicetree(PathBuf),
    Options(Vec<String>),
}

impl crate::BootFile for EntryKey {
    fn boot_file(&self) -> Option<&Path> {
        match self {
            EntryKey::Linux(path) => Some(path),
            EntryKey::Devicetree(path) => Some(path),
            EntryKey::Options(_) => None,
            EntryKey::Title(_) => None,
        }
    }
}

impl fmt::Display for EntryKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryKey::Linux(path) => write!(f, "linux {}", path.display()),
            EntryKey::Devicetree(path) => write!(f, "devicetree {}", path.display()),
            EntryKey::Options(options) => write!(f, "options {}", options.join(" ")),
            EntryKey::Title(title) => write!(f, "title {}", title),
        }
    }
}

impl FromStr for EntryKey {
    type Err = crate::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let (rest, entry) = parser::entry_key(input).map_err(Into::<Self::Err>::into)?;
        match rest {
            "" => Ok(entry),
            _ => Err(crate::Error::ErroneousEntry(format!(
                "trailing garbage: \"{}\"",
                rest
            ))),
        }
    }
}

/// A boot loader entry for this system. This datum represents a "Type #1" (text-based) boot entry
/// according to the [UAPI group Boot Loader Specification][https://uapi-group.org/specifications/specs/boot_loader_specification/]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize, ::serde::Serialize))]
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
    type Err = crate::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let (rest, entry) = parser::boot_entry(input).map_err(Into::<Self::Err>::into)?;
        match rest {
            "" => Ok(entry),
            _ => Err(crate::Error::ErroneousEntry(format!(
                "trailing garbage: \"{}\"",
                rest
            ))),
        }
    }
}

/// Modules and routines meant to aid deserializing UAPI bootloader entries using serde field
/// attributes.
#[cfg(feature = "serde")]
pub mod serde {
    /// Deserialize a boot entry using the [FromStr] implementation on [BootEntry].
    pub mod from_str {
        use crate::uapi;
        use serde::de;
        use std::str::FromStr;

        pub fn deserialize<'de, D>(deserializer: D) -> Result<uapi::BootEntry, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            let value: String = de::Deserialize::deserialize(deserializer)?;
            uapi::BootEntry::from_str(&value).map_err(de::Error::custom)
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
