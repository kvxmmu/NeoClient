use {
    anyhow::Result,

    tokio::{
        net::TcpStream,
        spawn,

        sync::mpsc::unbounded_channel
    },

    crate::{
        ext::*,
        pair::*,
        factory::*,
        producer::*,
        consumer::*,
    }
};

pub async fn run_tcp_connector<T>(
    stream: TcpStream,
    factory: T
) -> Result<()>
where T: PipelineFactory {
    let buffer_size = factory.buffer_size();
    let address = stream.local_addr()?;
    let (reader, writer) = stream.into_buffered_with_capacity(
        &buffer_size
    );

    let (sink, stream) = unbounded_channel();
    let ProducerConsumerPair { producer, consumer } = factory.create_producer_consumer(
        reader,
        writer,
        address,
    );

    spawn(async move { producer.run(sink).await.unwrap_or_default(); });
    spawn(async move { consumer.run(stream).await }).await?
} 
