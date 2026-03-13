use std::sync::Arc;

use anima_brain::AnthropicClient;
use anima_core::{Node, Result};
use anima_cortex::CortexRepo;

use crate::{cue, direct, scoring, spread};

/// Assembled context ready to be passed to the LLM.
#[derive(Debug, Clone)]
pub struct AssembledContext {
    pub nodes: Vec<ScoredNode>,
    pub total_tokens_estimate: usize,
}

#[derive(Debug, Clone)]
pub struct ScoredNode {
    pub node: Node,
    pub score: f64,
}

pub struct ContextPipeline {
    cortex: Arc<CortexRepo>,
    brain: Arc<AnthropicClient>,
}

impl ContextPipeline {
    pub fn new(cortex: Arc<CortexRepo>, brain: Arc<AnthropicClient>) -> Self {
        Self { cortex, brain }
    }

    /// Run the full context selection pipeline for a user input.
    pub async fn recall(&self, user_input: &str) -> Result<AssembledContext> {
        // Stage 1: Cue extraction
        let cues = cue::extract(&self.brain, user_input).await?;

        // Stage 2: Direct retrieval
        let mut candidates = direct::retrieve(&self.cortex, &cues).await?;

        // Stage 3: Spreading activation
        let seed_ids: Vec<String> = candidates.iter().map(|n| n.id.clone()).collect();
        if !seed_ids.is_empty() {
            let activated = spread::activate(&self.cortex, &seed_ids).await?;
            candidates.extend(activated);
        }

        // Stage 4: Semantic search (stub — Phase 2)
        // Stage 5: Scoring
        let scored = scoring::score(candidates, &cues);

        // Stage 6: Assembly (simple for now)
        let total_tokens_estimate = scored.iter().map(|s| estimate_tokens(&s.node)).sum();

        Ok(AssembledContext {
            nodes: scored,
            total_tokens_estimate,
        })
    }
}

fn estimate_tokens(node: &Node) -> usize {
    let text_len = node.title.len() + node.content.as_deref().map_or(0, |c| c.len());
    // Rough estimate: ~4 chars per token
    text_len / 4
}
