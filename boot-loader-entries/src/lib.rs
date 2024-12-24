use std::path::Path;

mod parser;

/// Definitions and logic for UAPI-Group Boot Loader Specification -compliant boot loader entries.
pub mod uapi;

/// Definitions and logic for Syslinux configurations
pub mod syslinux;

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

/// Trait to query associated file options on keys
pub trait BootFile {
    /// Request an associated file on a keyword/directive
    fn boot_file(&self) -> Option<&Path>;
}
