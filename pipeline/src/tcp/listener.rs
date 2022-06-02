use {
    crate::{
        factory::*,
        
        consumer::*,
        producer::*,

        ext::*,
        pair::*,
        Result,
    },

    tokio::{
        net::TcpListener,
        sync::mpsc::unbounded_channel,
        spawn,
    },
};

pub struct TcpNetworkPipelineListener<Factory> {
    listener: TcpListener,
    factory: Factory,
}

impl<Factory> TcpNetworkPipelineListener<Factory>
where Factory: PipelineFactory
{
    pub async fn run(self) -> Result<()> {
        let buffer_size = self.factory.buffer_size();

        loop {
            let (stream, address) = self.listener.accept().await?;
            let (reader, writer) = stream.into_buffered_with_capacity(
                &buffer_size
            );

            let (sink, stream) = unbounded_channel();
            let ProducerConsumerPair { consumer, producer } = self.factory.create_producer_consumer(
                reader,
                writer,
                address.clone()
            );

            spawn(async move { consumer.run(stream).await });
            spawn(async move { producer.run(sink).await });
        }
    }

    pub fn new(
        factory: Factory,
        listener: TcpListener,
    ) -> Result<Self> {
        Ok(Self { listener
                , factory })
    }
}
