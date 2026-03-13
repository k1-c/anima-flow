use anima_core::Node;

use crate::cue::Cues;
use crate::pipeline::ScoredNode;

/// Stage 5: Score and rank candidate nodes.
pub fn score(candidates: Vec<Node>, cues: &Cues) -> Vec<ScoredNode> {
    let mut scored: Vec<ScoredNode> = candidates
        .into_iter()
        .map(|node| {
            let s = compute_score(&node, cues);
            ScoredNode { node, score: s }
        })
        .collect();

    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Keep top N
    scored.truncate(20);
    scored
}

fn compute_score(node: &Node, cues: &Cues) -> f64 {
    let mut score = 0.0;

    // Title match bonus
    for entity in &cues.entities {
        if node.title.to_lowercase().contains(&entity.to_lowercase()) {
            score += 1.0;
        }
    }

    // Recency: Ebbinghaus-inspired decay
    let days_since = (chrono::Utc::now() - node.updated_at).num_days().max(0) as f64;
    let lambda = 0.1 / (1.0 + (node.access_count.max(0) as f64).ln_1p());
    let recency = (-lambda * days_since).exp();
    score += 0.6 * recency;

    // Frequency bonus
    let freq = (node.access_count as f64).ln_1p() * 0.1;
    score += freq;

    // Type bonus based on intent
    let type_bonus = match (cues.intent.as_str(), node.node_type.as_str()) {
        ("task_check", "space" | "daily") => 0.3,
        ("contact", "person" | "episode") => 0.3,
        ("procedure", "procedure") => 0.3,
        ("review", "episode" | "daily") => 0.3,
        ("decision", "decision" | "space") => 0.3,
        _ => 0.0,
    };
    score += type_bonus;

    score
}
