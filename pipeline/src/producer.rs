use {
    async_trait::async_trait,
    tokio::{
        sync::mpsc::UnboundedSender
    },

    crate::Result,
};

#[async_trait]
pub trait PipelineProducer {
    type Frame: Send + 'static;

    async fn run(
        self,
        sink: UnboundedSender<Self::Frame>
    ) -> Result<()>;
}
