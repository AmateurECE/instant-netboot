use std::{fs, path::PathBuf, str::FromStr};

use async_std::task::block_on;
use async_tftp::server::TftpServerBuilder;
use boot_loader_entries::uapi::BootEntry;
use clap::Parser;
use instant_netboot::NetbootServer;
use tracing::info;

mod instant_netboot;
mod tftp;

#[derive(clap::Parser)]
struct Args {
    /// The configuration file
    pub configuration: PathBuf,

    /// The address to listen on
    #[arg(short, long, default_value_t = String::from("0.0.0.0:6969"))]
    pub socket: String,

    /// Verbose logging
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_max_level(if args.verbose {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .with_writer(std::io::stderr)
        .init();

    info!("Serving configuration: {}", args.configuration.display());
    let configuration = BootEntry::from_str(&String::from_utf8(fs::read(args.configuration)?)?)?;
    let server = NetbootServer::new(configuration.try_into().unwrap());
    block_on(async {
        let addr = args.socket.parse()?;
        let tftpd = TftpServerBuilder::with_handler(tftp::TftpHandler { server })
            .bind(addr)
            .build()
            .await?;
        info!("Listening on {}", addr);
        tftpd.serve().await?;
        Ok(())
    })
}
