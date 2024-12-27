use std::{
    borrow::Cow,
    cell::LazyCell,
    net::IpAddr,
    path::{Path, PathBuf},
};

use async_std::fs::File;
use boot_loader_entries::{syslinux, BootFile};
use futures::AsyncRead;
use regex::Regex;
use serde::Deserialize;

/// The NFS version to configure the target for
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub enum NfsVersion {
    NFSv3,
    NFSv4,
}

/// The IP configuration for the target
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub enum TargetIpConfiguration {
    Dhcp,
    Static {},
}

/// NFS Configuration for instant-netboot
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub struct NfsConfiguration {
    /// The NFS host
    pub host: IpAddr,
    /// The NFS share to mount
    pub share: PathBuf,
    /// The NFS version to use
    pub version: NfsVersion,
    /// IP configuration for the target
    pub target_ip: TargetIpConfiguration,
    /// Whether the share should be mounted writable or not.
    pub is_writable: bool,
}

/// This netboot server is a "just add water" solution for netbooting Linux machines in
/// development.
#[derive(Debug)]
pub struct NetbootServer {
    // TODO: Make this configurable.
    configuration: syslinux::Label,
    nfs: Option<NfsConfiguration>,
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

fn make_nfsroot_option(nfs: &NfsConfiguration) -> String {
    let version = match nfs.version {
        NfsVersion::NFSv3 => "3",
        NfsVersion::NFSv4 => "4",
    };
    format!(
        "nfsroot={}:{},vers={},tcp",
        nfs.host,
        nfs.share.display(),
        version
    )
}

fn make_ip_option(config: &TargetIpConfiguration) -> String {
    // "ip=dhcp".to_string(),
    let spec = match config {
        TargetIpConfiguration::Dhcp => "dhcp",
        TargetIpConfiguration::Static {} => {
            // FIXME: Implement Static IP configuration
            panic!("Static IP configuration is not currently implemented")
        }
    };
    format!("ip={}", spec)
}

/// Update the configuration with NFS parameters
fn make_nfs_configuration(
    mut configuration: syslinux::Label,
    nfs: &NfsConfiguration,
) -> syslinux::Label {
    let mut nfs_args = vec![
        "root=/dev/nfs".to_string(),
        if nfs.is_writable {
            "rw".to_string()
        } else {
            "ro".to_string()
        },
        make_nfsroot_option(nfs),
        "rootwait".to_string(),
        make_ip_option(&nfs.target_ip),
    ];

    // Have to find the existing APPEND directive, if it exists
    if let Some(options) = configuration
        .directives
        .iter_mut()
        .find(|k| matches!(k, syslinux::LabelDirective::Append(_)))
    {
        let syslinux::LabelDirective::Append(ref mut current_args) = options else {
            // INVARIANT: We just sought the Append() directive.
            unreachable!()
        };
        current_args.append(&mut nfs_args);
    }
    // Otherwise, add an APPEND directive
    else {
        configuration
            .directives
            .push(syslinux::LabelDirective::Append(nfs_args));
    }
    configuration
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
        Self {
            configuration,
            nfs: None,
        }
    }

    pub fn with_nfs(configuration: syslinux::Label, nfs: NfsConfiguration) -> Self {
        Self {
            configuration,
            nfs: Some(nfs),
        }
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
            let configuration = if let Some(nfs) = &self.nfs {
                Cow::Owned(make_nfs_configuration(self.configuration.clone(), nfs))
            } else {
                Cow::Borrowed(&self.configuration)
            };

            return Ok(Box::new(futures::io::Cursor::new(
                configuration.to_string(),
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
