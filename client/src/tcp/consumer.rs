use {
    pipeline::{
        consumer::*,
        ext::*,

        async_trait,
        Result,
    },

    neogrok_codec::{
        protocol::prelude::*,
        writer::*,
    },

    tokio::{
        sync::mpsc::UnboundedReceiver
    },
    std::net::SocketAddr,

    crate::{
        compression::*,
    }
};

pub struct TcpConsumer {
    writer: Writer,
    address: SocketAddr,

    compression: Compression,

    magic: Option<String>,
    port: u16,
}

#[async_trait]
impl PipelineConsumer for TcpConsumer {
    type Frame = NFrame;

    async fn run(
        mut self,
        mut stream: UnboundedReceiver<Self::Frame>
    ) -> Result<()> {
        let mut codec = CodecWriter::new(
            &mut self.writer,
            self.compression.level,
            self.compression.profit,
            self.compression.threshold
        );
        codec.write_ping_request().await?;

        if let Some(ref magic) = self.magic {
            codec.write_authorize(
                magic
            ).await?;
        }

        loop {
            let frame = if let Some(fr) = stream.recv().await {
                fr
            } else {
                break;
            };

            match frame {
                NFrame::PingResponse { name } => {
                    log::info!("Ping received: {}", name);
                },

                NFrame::Error { code } => {
                    log::info!("An error occurred: {}", code.to_error_string());
                },

                frame => {
                    log::debug!("Unhandled frame: {:?}", frame);
                }
            }
        }

        Ok(())
    }
}

impl TcpConsumer {
    pub fn new(
        writer: Writer,
        address: SocketAddr,

        port: u16,

        magic: Option<String>,
        compression: Compression,
    ) -> Self {
        Self { writer
             , address
             , compression
             , port
             , magic }
    }
}

impl Drop for TcpConsumer {
    fn drop(&mut self) {
        log::debug!("Consumer dropped");
    }
}
