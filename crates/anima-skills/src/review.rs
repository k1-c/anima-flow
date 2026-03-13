use anima_core::{Category, NewNode, NodeType, NodeUpdate, Result};
use async_trait::async_trait;

use crate::traits::{Skill, SkillContext};

pub struct ReviewSkill;

#[async_trait]
impl Skill for ReviewSkill {
    fn name(&self) -> &str {
        "review"
    }

    fn description(&self) -> &str {
        "End-of-day review: summarize, update DailyNote, consolidate memory"
    }

    async fn execute(&self, ctx: &SkillContext, _args: &str) -> Result<()> {
        ctx.gateway.send("🌆 Starting daily review...").await?;

        // 1. Observe: collect today's activities from connectors
        let mut activities = Vec::new();
        for query in &ctx.queries {
            match query.fetch_inbox().await {
                Ok(items) => {
                    for item in items {
                        activities.push(format!(
                            "- [{}] {}: {}",
                            item.source,
                            item.title,
                            item.content.chars().take(100).collect::<String>()
                        ));
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        source = query.source_name(),
                        error = %e,
                        "connector fetch failed"
                    );
                }
            }
        }

        // Include processed inbox items from today
        let today_inbox = ctx.cortex.find_by_type(NodeType::Inbox).await?;
        for node in &today_inbox {
            let status = node
                .metadata
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            activities.push(format!("- [inbox/{}] {}", status, node.title));
        }

        let activities_text = if activities.is_empty() {
            "No activities captured today.".to_string()
        } else {
            activities.join("\n")
        };

        // 2. Internalize: recall context
        let context = ctx
            .context_engine
            .recall("daily review today summary")
            .await?;

        let context_summary = context
            .nodes
            .iter()
            .take(10)
            .map(|n| format!("- [{}] {}", n.node.node_type, n.node.title))
            .collect::<Vec<_>>()
            .join("\n");

        // 3. Brain: synthesize review
        let review = ctx
            .brain
            .synthesize_review(&activities_text, &context_summary)
            .await?;

        // 4. Intervene: present review
        let mut output = String::new();
        output.push_str(&format!("**Review:** {}\n\n", review.summary));

        if !review.completed.is_empty() {
            output.push_str("**Completed:**\n");
            for item in &review.completed {
                output.push_str(&format!("  ✅ {item}\n"));
            }
        }

        if !review.in_progress.is_empty() {
            output.push_str("\n**In Progress:**\n");
            for item in &review.in_progress {
                output.push_str(&format!("  🔄 {item}\n"));
            }
        }

        if !review.unstarted.is_empty() {
            output.push_str("\n**Not Started:**\n");
            for item in &review.unstarted {
                output.push_str(&format!("  ⬜ {item}\n"));
            }
        }

        if !review.tomorrow.is_empty() {
            output.push_str("\n**Tomorrow's Priorities:**\n");
            for item in &review.tomorrow {
                output.push_str(&format!("  → {item}\n"));
            }
        }

        if !review.learnings.is_empty() {
            output.push_str("\n**Learnings:**\n");
            for item in &review.learnings {
                output.push_str(&format!("  💡 {item}\n"));
            }
        }

        ctx.gateway.send(&output).await?;

        // 5. Internalize: update DailyNote
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let daily_content = format!(
            "## Review\n{}\n\n## Completed\n{}\n\n## Tomorrow\n{}",
            review.summary,
            review.completed.join("\n- "),
            review.tomorrow.join("\n- "),
        );

        let existing = ctx.cortex.find_by_title(&today).await?;
        if let Some(daily) = existing.iter().find(|n| n.node_type == "daily") {
            // Append review to existing DailyNote
            let updated_content = format!(
                "{}\n\n---\n\n{}",
                daily.content.as_deref().unwrap_or(""),
                daily_content
            );
            ctx.cortex
                .update_node(
                    &daily.id,
                    &NodeUpdate {
                        content: Some(updated_content),
                        ..Default::default()
                    },
                )
                .await?;
        } else {
            ctx.cortex
                .insert_node(&NewNode {
                    node_type: NodeType::Daily,
                    category: Category::Memory,
                    title: today,
                    content: Some(daily_content),
                    metadata: serde_json::json!({}),
                })
                .await?;
        }

        // 6. Memory consolidation: promote important episodes to long-term
        // (Simplified: log learnings as learning Nodes)
        for learning in &review.learnings {
            ctx.cortex
                .insert_node(&NewNode {
                    node_type: NodeType::Learning,
                    category: Category::Knowledge,
                    title: learning.clone(),
                    content: Some(format!(
                        "Learned on {}: {learning}",
                        chrono::Utc::now().format("%Y-%m-%d")
                    )),
                    metadata: serde_json::json!({"source": "daily_review"}),
                })
                .await?;
        }

        Ok(())
    }
}
