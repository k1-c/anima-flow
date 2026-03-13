use anima_core::Result;
use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::traits::Gateway;

pub struct CliGateway;

impl CliGateway {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CliGateway {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Gateway for CliGateway {
    async fn send(&self, message: &str) -> Result<()> {
        println!("{message}");
        Ok(())
    }

    async fn receive(&self) -> Result<String> {
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .await
            .map_err(|e| anima_core::Error::Other(anyhow::anyhow!("stdin read error: {e}")))?;
        Ok(line.trim().to_string())
    }
}
