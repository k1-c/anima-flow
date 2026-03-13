use anima_core::Result;
use async_trait::async_trait;

/// Trait for user-facing gateways (intervention layer).
#[async_trait]
pub trait Gateway: Send + Sync {
    /// Send a message to the user.
    async fn send(&self, message: &str) -> Result<()>;

    /// Receive input from the user (blocking until input arrives).
    async fn receive(&self) -> Result<String>;

    /// Send a message and wait for a response.
    async fn ask(&self, question: &str) -> Result<String> {
        self.send(question).await?;
        self.receive().await
    }
}
