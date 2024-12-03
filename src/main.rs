use async_tftp::server::TftpServerBuilder;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(std::io::stderr)
        .init();
    let tftpd = TftpServerBuilder::with_dir_ro(".")?
        .bind("[::1]:6969".parse()?)
        .build()
        .await?;
    tftpd.serve().await?;
    Ok(())
}
