use std::net::SocketAddr;

use boot_loader_entries::uapi;
use serde::Deserialize;

use crate::instant_netboot::NfsConfiguration;

fn default_socket() -> SocketAddr {
    "0.0.0.0:6969".parse().unwrap()
}

#[derive(Deserialize)]
pub struct NetbootConfiguration {
    #[serde(default = "default_socket")]
    pub socket: SocketAddr,
    #[serde(deserialize_with = "uapi::serde::from_str::deserialize")]
    pub pxe: uapi::BootEntry,
}

#[derive(Deserialize)]
pub struct Configuration {
    pub tftp: NetbootConfiguration,
    pub nfs: Option<NfsConfiguration>,
}
