use std::{net::SocketAddr, path::Path};

use async_std::task::block_on;
use async_tftp::{packet, server::TftpServerBuilder};
use boot_loader_entries::{BootEntry, EntryKey};
use futures::AsyncRead;
use instant_netboot::NetbootServer;
use tracing::info;

mod instant_netboot;

/// Adapter for async_tftp
struct TftpHandler {
    server: NetbootServer,
}

impl From<instant_netboot::Error> for packet::Error {
    fn from(value: instant_netboot::Error) -> Self {
        match value {
            instant_netboot::Error::InvalidRequestPath => {
                packet::Error::Msg("Failed to parse request path".to_string())
            }
            instant_netboot::Error::FileNotFound => packet::Error::FileNotFound,
            instant_netboot::Error::IoError => packet::Error::Msg("I/O error".to_string()),
        }
    }
}

#[async_trait::async_trait]
impl async_tftp::server::Handler for TftpHandler {
    type Reader = Box<dyn AsyncRead + Send + Unpin + 'static>;
    type Writer = futures::io::Sink;

    async fn read_req_open(
        &mut self,
        _client: &SocketAddr,
        path: &Path,
    ) -> Result<(Self::Reader, Option<u64>), packet::Error> {
        Ok((self.server.tftp_get(path).await?, None))
    }

    async fn write_req_open(
        &mut self,
        _client: &SocketAddr,
        _path: &Path,
        _size: Option<u64>,
    ) -> Result<Self::Writer, packet::Error> {
        Err(packet::Error::IllegalOperation)
    }
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_writer(std::io::stderr)
        .init();

    let configuration = BootEntry {
        keys: vec![EntryKey::Linux("stereo-gadget/Image".into())],
    };
    let server = NetbootServer::new(configuration);
    block_on(async {
        let addr = "[::1]:6969".parse()?;
        let tftpd = TftpServerBuilder::with_handler(TftpHandler { server })
            .bind(addr)
            .build()
            .await?;
        info!("Listening on {}", addr);
        tftpd.serve().await?;
        Ok(())
    })
}
