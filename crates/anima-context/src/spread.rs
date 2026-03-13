use std::sync::Arc;

use anima_core::{Node, Result};
use anima_cortex::CortexRepo;

/// Stage 3: Spreading activation through the Edge graph.
pub async fn activate(cortex: &Arc<CortexRepo>, seed_ids: &[String]) -> Result<Vec<Node>> {
    let activated =
        anima_cortex::graph::spreading_activation(cortex.pool(), seed_ids, 3, 0.7, 20).await?;

    Ok(activated.into_iter().map(|a| a.node).collect())
}
