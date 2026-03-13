use anima_core::{Category, NewNode, NodeType, Result};
use async_trait::async_trait;

use crate::traits::{Skill, SkillContext};

pub struct BriefingSkill;

#[async_trait]
impl Skill for BriefingSkill {
    fn name(&self) -> &str {
        "briefing"
    }

    fn description(&self) -> &str {
        "Briefing: calendar, inbox, tasks, and context"
    }

    async fn execute(&self, ctx: &SkillContext, _args: &str) -> Result<()> {
        ctx.gateway
            .send("📋 Preparing your briefing...")
            .await?;

        // 1. Observe: collect inbox from all connectors
        let mut all_items = Vec::new();
        for query in &ctx.queries {
            match query.fetch_inbox().await {
                Ok(items) => all_items.extend(items),
                Err(e) => {
                    tracing::warn!(
                        source = query.source_name(),
                        error = %e,
                        "connector fetch failed"
                    );
                }
            }
        }

        // 2. Internalize: recall relevant context
        let context = ctx
            .context_engine
            .recall("briefing today schedule tasks")
            .await?;

        let context_summary: String = context
            .nodes
            .iter()
            .take(10)
            .map(|n| {
                format!(
                    "- [{}] {}: {}",
                    n.node.node_type,
                    n.node.title,
                    n.node
                        .content
                        .as_deref()
                        .unwrap_or("")
                        .chars()
                        .take(200)
                        .collect::<String>()
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let inbox_summary: String = if all_items.is_empty() {
            "No new inbox items.".to_string()
        } else {
            all_items
                .iter()
                .map(|item| {
                    format!(
                        "- [{}] {}: {}",
                        item.source,
                        item.title,
                        item.content.chars().take(100).collect::<String>()
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        };

        // 3. Brain: synthesize briefing
        let briefing = ctx
            .brain
            .synthesize_briefing(&inbox_summary, &context_summary)
            .await?;

        // 4. Intervene: send briefing to user
        let mut output = String::new();
        output.push_str(&briefing.greeting);
        output.push_str("\n\n");
        output.push_str(&briefing.summary);

        if !briefing.priorities.is_empty() {
            output.push_str("\n\n**Priorities:**\n");
            for p in &briefing.priorities {
                output.push_str(&format!("  - {p}\n"));
            }
        }

        if !briefing.reminders.is_empty() {
            output.push_str("\n**Reminders:**\n");
            for r in &briefing.reminders {
                output.push_str(&format!("  - {r}\n"));
            }
        }

        ctx.gateway.send(&output).await?;

        // 5. Internalize: create/update today's DailyNote
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let daily_content = format!(
            "## Briefing\n{}\n\n## Inbox ({} items)\n{}\n\n## Priorities\n{}",
            briefing.summary,
            all_items.len(),
            inbox_summary,
            briefing.priorities.join("\n- "),
        );

        let existing = ctx.cortex.find_by_title(&today).await?;
        let has_daily = existing.iter().any(|n| n.node_type == "daily");

        if !has_daily {
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

        Ok(())
    }
}
