use std::{cell::LazyCell, path::Path};

use async_std::fs::File;
use boot_loader_entries::{syslinux, BootFile};
use futures::AsyncRead;
use regex::Regex;

/// This netboot server is a "just add water" solution for netbooting Linux machines in
/// development.
#[derive(Debug)]
pub struct NetbootServer {
    // TODO: Make this configurable.
    configuration: syslinux::Label,
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
    let Ok(path) = path.strip_prefix(Path::new("pxelinux.cfg")) else {
        return Ok(false);
    };
    let path = path.to_str().ok_or(Error::InvalidRequestPath)?;

    // An UUID
    const UUID: LazyCell<Regex> = LazyCell::new(|| {
        Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$").unwrap()
    });
    // A hyphen-separated MAC address prefixed by 01 (this is the medium type--01 is Ethernet)
    const MAC_ADDRESS: LazyCell<Regex> =
        LazyCell::new(|| Regex::new(r"^01-([0-9a-f]{2}-){5}[0-9a-f]{2}$").unwrap());
    // An IP address encoded in hexadecimal
    const IP_ADDRESS: LazyCell<Regex> = LazyCell::new(|| Regex::new(r"^[A-F0-9]{1,8}$").unwrap());
    Ok(UUID.is_match(path) || MAC_ADDRESS.is_match(path) || IP_ADDRESS.is_match(path))
}

/// Get the list of files mentioned in this boot entry.
fn listed_files<'a>(label: &'a syslinux::Label) -> impl Iterator<Item = &'a Path> {
    label
        .directives
        .iter()
        .filter_map(|key| key.boot_file())
        // TODO: Unwrap here
        .chain([label.kernel.boot_file().unwrap()])
}

impl NetbootServer {
    pub fn new(configuration: syslinux::Label) -> Self {
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
