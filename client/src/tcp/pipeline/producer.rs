use {
    pipeline::{
        producer::*,
        ext::*,

        async_trait,
        Result,
    },

    tokio::{
        sync::mpsc::UnboundedSender
    },

    neogrok_codec::{
        protocol::prelude::*,
        reader::*,
    },

    std::net::SocketAddr,
};

pub struct TcpProducer {
    reader: Reader,
    address: SocketAddr,
}

#[async_trait]
impl PipelineProducer for TcpProducer {
    type Frame = NFrame;

    async fn run(
        mut self,
        sink: UnboundedSender<Self::Frame>
    ) -> Result<()> {
        let mut codec = CodecReader::client(
            &mut self.reader
        );
        loop {
            let frame = codec.read_frame().await?;
            sink.send(frame)?;
        }
    }
}

impl TcpProducer {
    pub fn new(
        reader: Reader,
        address: SocketAddr,
    ) -> Self {
        Self { reader
             , address }
    }
}

impl Drop for TcpProducer {
    fn drop(&mut self) {
        log::info!("Disconnected from the main server ({})", self.address);
        log::debug!("Main producer dropped");
    }
}

