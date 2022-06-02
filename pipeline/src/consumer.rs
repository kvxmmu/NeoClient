use {
    async_trait::async_trait,
    tokio::{
        sync::mpsc::UnboundedReceiver
    },

    anyhow::Result,
};

#[async_trait]
pub trait PipelineConsumer {
    type Frame: Send + 'static;

    async fn run(
        self,
        stream: UnboundedReceiver<Self::Frame>
    ) -> Result<()>;
}
