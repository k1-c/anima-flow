use std::sync::Arc;

use anima_brain::AnthropicClient;
use anima_core::Result;

// Re-export CueExtraction from brain for use in other context modules.
pub use anima_brain::client::CueExtraction as Cues;

/// Stage 1: Extract cues from user input using the LLM.
pub async fn extract(brain: &Arc<AnthropicClient>, user_input: &str) -> Result<Cues> {
    brain.extract_cues(user_input).await
}
