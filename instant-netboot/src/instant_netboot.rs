use std::{cell::LazyCell, path::Path};

use async_std::fs::File;
use boot_loader_entries::BootEntry;
use futures::AsyncRead;
use regex::Regex;

/// This netboot server is a "just add water" solution for netbooting Linux machines in
/// development.
pub struct NetbootServer {
    configuration: BootEntry,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("request path is invalid")]
    InvalidRequestPath,
    #[error("no such file or directory")]
    FileNotFound,
    #[error("I/O error")]
    IoError,
}

/// Returns Ok(true) if the path is for a PXE configuration file. Returns Err if the path is
/// invalid.
fn is_pxe_config_path(path: &Path) -> Result<bool, Error> {
    const PXE_PATH_PATTERN: LazyCell<Regex> =
        LazyCell::new(|| Regex::new(r"^pxelinux\.cfg/[A-F0-9]{8}$").unwrap());
    Ok(PXE_PATH_PATTERN.is_match(path.to_str().ok_or(Error::InvalidRequestPath)?))
}

/// Get the list of files mentioned in this boot entry.
fn listed_files<'a>(boot_entry: &'a BootEntry) -> impl Iterator<Item = &'a Path> {
    boot_entry.keys.iter().filter_map(|key| key.file())
}

impl NetbootServer {
    pub fn new(configuration: BootEntry) -> Self {
        Self { configuration }
    }

    /// Route a TFTP GET request to this server. If the path refers to a PXE configuration, the
    /// configuration is generated. If it refers to a boot file, the file is served, etc.
    pub async fn tftp_get(
        &mut self,
        path: &Path,
    ) -> Result<Box<dyn AsyncRead + Send + Unpin + 'static>, Error> {
        // If it's pxelinux.cfg/C0A802BA (or if it matches that pattern) generate a boot
        // configuration and return that.
        if is_pxe_config_path(path)? {
            return Ok(Box::new(futures::io::Cursor::new(
                self.configuration.to_string(),
            )));
        }

        // Otherwise, if it's a path to a file that we are serving (a boot file), serve it!
        match listed_files(&self.configuration)
            .find(|file| *file == path)
            .ok_or(Error::FileNotFound)
        {
            Ok(file) => Ok(Box::new(
                File::open(file).await.map_err(|_| Error::IoError)?,
            )),
            Err(_) => Err(Error::FileNotFound),
        }
    }
}
