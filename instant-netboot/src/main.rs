use std::{fs::File, path::PathBuf};

use async_std::task::block_on;
use async_tftp::server::TftpServerBuilder;
use clap::Parser;
use instant_netboot::NetbootServer;
use tracing::info;

mod config;
mod instant_netboot;
mod tftp;

#[derive(clap::Parser)]
struct Args {
    /// The configuration file
    pub configuration: PathBuf,

    /// Verbose logging
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config: config::Configuration = serde_yaml::from_reader(File::open(args.configuration)?)?;

    tracing_subscriber::fmt()
        .with_max_level(if args.verbose {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .with_writer(std::io::stderr)
        .init();

    let boot_configuration = config.tftp.pxe.try_into().unwrap();
    let server = match config.nfs {
        Some(nfs) => NetbootServer::with_nfs(boot_configuration, nfs),
        None => NetbootServer::new(boot_configuration),
    };
    block_on(async {
        let tftpd = TftpServerBuilder::with_handler(tftp::TftpHandler { server })
            .bind(config.tftp.socket)
            .build()
            .await?;
        info!("Listening on {}", config.tftp.socket);
        tftpd.serve().await?;
        Ok(())
    })
}
