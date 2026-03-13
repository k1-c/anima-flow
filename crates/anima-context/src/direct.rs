use std::sync::Arc;

use anima_core::{Node, Result};
use anima_cortex::CortexRepo;

use crate::cue::Cues;

/// Stage 2: Direct retrieval from Cortex based on extracted cues.
pub async fn retrieve(cortex: &Arc<CortexRepo>, cues: &Cues) -> Result<Vec<Node>> {
    let mut results = Vec::new();

    // Search by entity names
    for entity in &cues.entities {
        let nodes = cortex.find_by_title(entity).await?;
        results.extend(nodes);
    }

    // Search by topics via FTS
    for topic in &cues.topics {
        let nodes = cortex.full_text_search(topic).await?;
        results.extend(nodes);
    }

    // Deduplicate by node id
    results.sort_by(|a, b| a.id.cmp(&b.id));
    results.dedup_by(|a, b| a.id == b.id);

    Ok(results)
}
