use {
    crate::{
        compression::*,
        tcp::factory::*,
    },

    pipeline::{
        tcp::{
            connector::*,
        },

        ext::*,
        Result,
    },

    tokio::{
        net::TcpStream,
    }
};

pub async fn run_tcp_client(
    local: String,
    remote: String,

    magic: Option<String>,

    port: u16,
    buffer_size: BufferSize,
    compression: Compression,
    synchronize: bool,
) -> Result<()> {
    log::info!("Connecting to the {}...", remote);

    let factory = TcpFactory::new(
        buffer_size,
        compression,
        port,
        magic,
        local,
        remote.clone(),
        !synchronize
    );
    let stream = TcpStream::connect(&remote).await?;
    log::debug!("Connected");

    run_tcp_connector(
        stream,
        factory
    ).await
}
