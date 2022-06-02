use {
    async_trait::async_trait,
    std::net::SocketAddr,

    crate::{
        consumer::*,
        producer::*,
        ext::*,
        pair::*,
    }
};

pub trait PipelineFactory {
    type Consumer: PipelineConsumer + Send + 'static;
    type Producer: PipelineProducer<Frame = <Self::Consumer as PipelineConsumer>::Frame> + Send + 'static;

    fn create_producer_consumer(
        &self,
        reader: Reader,
        writer: Writer,
        address: SocketAddr,
    ) -> ProducerConsumerPair<Self::Producer, Self::Consumer>;

    fn buffer_size(&self) -> BufferSize;
}
