use core::fmt;
use std::path::{Path, PathBuf};

/// A menu entry key, containing a fragment of configuration for the boot loader.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EntryKey {
    Linux(PathBuf),
    Devicetree(PathBuf),
}

impl EntryKey {
    pub fn file<'a>(&'a self) -> Option<&'a Path> {
        match self {
            EntryKey::Linux(path) => Some(path),
            EntryKey::Devicetree(path) => Some(path),
        }
    }
}

impl fmt::Display for EntryKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryKey::Linux(path) => write!(f, "linux {}", path.display()),
            EntryKey::Devicetree(path) => write!(f, "devicetree {}", path.display()),
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
