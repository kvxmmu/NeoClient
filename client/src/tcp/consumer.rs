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
        sync::mpsc::{ UnboundedReceiver
                    , UnboundedSender
                    , unbounded_channel },

        spawn,
        select,
    },
    std::{ net::SocketAddr },

    crate::{
        compression::*,
        tcp::proxy::{
            frame::*,
            clients::*,
            listener::*,
        }
    },

    anyhow::anyhow,
};

pub struct TcpConsumer {
    pub writer: CodecWriter<Writer>,
    pub address: SocketAddr,

    pub compression: Compression,
    pub use_overrides: bool,

    pub magic: Option<String>,
    pub port: u16,

    pub local: String,
    pub remote: String,

    pub clients: ProxyClients,
    pub authorized: bool,

    pub buffer_size: usize,
}

impl TcpConsumer {
    async fn handle_network(
        &mut self,
        frame: NFrame,
        sink: &UnboundedSender<MasterFrame>
    ) -> Result<()> {
        match frame {
            NFrame::Connect { id } => {
                let (slave_sink, slave_stream) = unbounded_channel();

                self.clients.connected(id, slave_sink);
                spawn(run_tcp_proxy_listener(
                    self.local.clone(),
                    slave_stream,
                    sink.clone(),
                    id,
                    self.buffer_size
                ));
            },

            NFrame::Forward { id, buffer } => {
                self.clients.forward(id, buffer);
            },

            NFrame::Disconnect { id } => {
                self.clients.disconnect(id);
            },

            //

            NFrame::ServerResponse { mut host, port } => {
                if &host == "0.0.0.0" {
                    host = self.remote.clone();
                }

                println!();
                log::info!("NeoGrok {}", env!("CARGO_PKG_VERSION"));
                log::info!("Listening on {}:{}", host, port);
                println!();
            },

            NFrame::SyncResponse {
                threshold,
                level,
                profit
            } => {
                self.writer.replace_cctx(
                    threshold as usize,
                    profit,
                    level as i32
                );

                println!();
                log::info!("Using server compression settings:");
                log::info!("    Algorithm: ZStandard");
                log::info!("    Level: {}", level);
                log::info!("    Profit: {}%", profit);
                log::info!("    Threshold: {}", threshold);

            },

            NFrame::UpdateRights { new_rights } => {
                log::debug!("Received rights: {}", new_rights.show_rights());

                self.authorized = true;
                if !self.use_overrides {
                    self.writer.write_sync_request().await?;
                }

                self.request_server().await?;
            },

            NFrame::PingResponse { name } => {
                log::info!("Server {}", name);

                if self.authorized {
                    
                    if !self.use_overrides {
                        self.writer.write_sync_request().await?;
                    }

                    self.request_server().await?;
                }
            },

            NFrame::Error { code } => {
                log::error!("An error occurred: {}", code.to_error_string());

                if code == ACCESS_DENIED {
                    return Err(anyhow!("Access denied"));
                }
            },

            frame => {
                log::debug!("Unhandled frame: {:?}", frame);
            }
        }

        Ok(())
    }

    async fn handle_slave(
        &mut self,
        frame: MasterFrame
    ) -> Result<()> {
        match frame {
            MasterFrame::Disconnected { id } => {
                self.writer.write_disconnect(id)
                           .await?;
                self.clients.disconnect(id);
            },

            MasterFrame::Forward { id, buffer } => {
                self.writer.write_forward(
                    id,
                    buffer
                ).await?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl PipelineConsumer for TcpConsumer {
    type Frame = NFrame;

    async fn run(
        mut self,
        mut network_stream: UnboundedReceiver<Self::Frame>
    ) -> Result<()> {
        self.writer.write_ping_request().await?;

        if let Some(ref magic) = self.magic {
            self.writer.write_authorize(
                magic
            ).await?;

            self.authorized = false;
        }

        let (sink, mut stream) = unbounded_channel();

        loop {
            select! {
                network_frame = network_stream.recv() => if let Some(frame) = network_frame {
                    self.handle_network(frame, &sink)
                        .await?;
                } else {
                    break;
                },

                slave_frame = stream.recv() => if let Some(frame) = slave_frame {
                    self.handle_slave(frame)
                        .await?;
                } else {
                    break;
                }
            }
            
        }

        Ok(())
    }
}

impl TcpConsumer {
    pub async fn request_server(
        &mut self,
    ) -> std::io::Result<()> {
        self.writer.write_request_server(
            TransportProtocol::Tcp,
            self.port
        ).await
    }
}

impl Drop for TcpConsumer {
    fn drop(&mut self) {
        log::debug!("Consumer dropped");
    }
}
