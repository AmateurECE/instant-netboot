use std::{net::SocketAddr, path::Path};

use async_tftp::packet;
use futures::AsyncRead;

use crate::instant_netboot;

/// Adapter for async_tftp
pub(crate) struct TftpHandler {
    pub server: instant_netboot::NetbootServer,
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
        client: &SocketAddr,
        path: &Path,
    ) -> Result<(Self::Reader, Option<u64>), packet::Error> {
        tracing::debug!("{}: GET {}", client, path.display());
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
