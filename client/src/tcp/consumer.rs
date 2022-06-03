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
    pub writer: Writer,
    pub address: SocketAddr,

    pub compression: Compression,
    pub use_overrides: bool,

    pub magic: Option<String>,
    pub port: u16,

    pub local: String,
}

#[async_trait]
impl PipelineConsumer for TcpConsumer {
    type Frame = NFrame;

    async fn run(
        mut self,
        mut stream: UnboundedReceiver<Self::Frame>
    ) -> Result<()> {
        let mut created_server = false;
        let mut authorized = true;

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

            authorized = false;
        }

        loop {
            let frame = if let Some(f) = stream.recv().await {
                f
            } else {
                break;
            };

            match frame {
                NFrame::SyncResponse {
                    threshold,
                    level,
                    profit
                } => {
                    codec.replace_cctx(
                        threshold as usize,
                        profit,
                        level as i32
                    );

                    log::info!("Using server compression settings:");
                    log::info!("    Algorithm: ZStandard");
                    log::info!("    Level: {}", level);
                    log::info!("    Profit: {}%", profit);
                    log::info!("    Threshold: {}", threshold);
                },

                NFrame::UpdateRights { new_rights } => {
                    log::info!("Received rights: {}", new_rights.show_rights());

                    authorized = true;
                    if !self.use_overrides {
                        codec.write_sync_request().await?;
                    }
                },

                NFrame::PingResponse { name } => {
                    log::info!("Ping received: {}", name);

                    if authorized {
                        codec.write_sync_request().await?;
                    }
                },

                NFrame::Error { code } => {
                    log::info!("An error occurred: {}", code.to_error_string());

                    if code == ACCESS_DENIED {
                        break;
                    }
                },

                frame => {
                    log::debug!("Unhandled frame: {:?}", frame);
                }
            }
        }

        Ok(())
    }
}

impl Drop for TcpConsumer {
    fn drop(&mut self) {
        log::debug!("Consumer dropped");
    }
}
