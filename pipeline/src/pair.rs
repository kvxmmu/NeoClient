use crate::{producer::*, consumer::*};

pub struct ProducerConsumerPair<P: PipelineProducer, C: PipelineConsumer> {
    pub producer: P,
    pub consumer: C,
}
