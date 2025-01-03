use std::io;
use std::path::PathBuf;

/// An id that uniquely identifies a file.
pub type FileId = u64;

/// The type of a file.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FileType {
    Regular,
    Directory,
    Link,
}

/// Filesystem-independent file metadata.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Metadata {
    pub file_type: FileType,
    pub mode: u32,
    pub uid: u64,
    pub gid: u64,
    pub mtime: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum FileError {
    #[error("not found")]
    NotFound,
    #[error("I/O")]
    Io(io::Error),
}

impl From<io::Error> for FileError {
    fn from(value: io::Error) -> Self {
        FileError::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct File {
    pub parent: Option<FileId>,
    pub attributes: Metadata,
    pub link_name: Option<PathBuf>,
    pub path: PathBuf,
}
