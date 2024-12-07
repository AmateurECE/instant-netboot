use std::{fs, path::PathBuf, str::FromStr};

use async_std::task::block_on;
use async_tftp::server::TftpServerBuilder;
use boot_loader_entries::BootEntry;
use clap::Parser;
use instant_netboot::NetbootServer;
use tracing::info;

mod instant_netboot;
mod tftp;

#[derive(clap::Parser)]
struct Args {
    /// The configuration file
    pub configuration: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_writer(std::io::stderr)
        .init();

    info!("Serving configuration: {}", args.configuration.display());
    let configuration = BootEntry::from_str(&String::from_utf8(fs::read(args.configuration)?)?)?;
    let server = NetbootServer::new(configuration);
    block_on(async {
        let addr = "[::1]:6969".parse()?;
        let tftpd = TftpServerBuilder::with_handler(tftp::TftpHandler { server })
            .bind(addr)
            .build()
            .await?;
        info!("Listening on {}", addr);
        tftpd.serve().await?;
        Ok(())
    })
}
