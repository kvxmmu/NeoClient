use {
    pipeline::{
        factory::*,
        ext::*,
        pair::*,
    },

    crate::{
        tcp::{
            consumer::*,
            producer::*,
        },

        compression::*,
    },

    std::net::SocketAddr,
};

pub struct TcpFactory {
    buffer_size: BufferSize,
    compression: Compression,

    port: u16,
    magic: Option<String>,

    local: String,
}

impl PipelineFactory for TcpFactory {
    type Consumer = TcpConsumer;
    type Producer = TcpProducer;

    fn create_producer_consumer(
        &self,
        reader: Reader,
        writer: Writer,
        address: SocketAddr,
    ) -> ProducerConsumerPair<Self::Producer, Self::Consumer> {
        ProducerConsumerPair {
            producer: TcpProducer::new(reader, address.clone()),
            consumer: TcpConsumer::new(
                writer,
                address,
                self.port,
                self.magic.clone(),
                self.compression.clone()
            ),
        }
    }

    fn buffer_size(&self) -> BufferSize {
        BufferSize { read: self.buffer_size.read
                   , write: self.buffer_size.write }
    }
}

impl TcpFactory {
    pub fn new(
        buffer_size: BufferSize,
        compression: Compression,

        port: u16,
        magic: Option<String>,

        local: String
    ) -> Self {
        Self { buffer_size
             , compression
             , port
             , magic
             , local }
    }
}
