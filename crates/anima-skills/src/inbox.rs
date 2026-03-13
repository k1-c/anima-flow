use anima_core::{Category, NewEdge, NewNode, NodeType, Result};
use async_trait::async_trait;

use crate::traits::{Skill, SkillContext};

pub struct InboxSkill;

#[async_trait]
impl Skill for InboxSkill {
    fn name(&self) -> &str {
        "inbox"
    }

    fn description(&self) -> &str {
        "Process inbox: collect, dedup, classify, and organize"
    }

    async fn execute(&self, ctx: &SkillContext, _args: &str) -> Result<()> {
        ctx.gateway.send("📥 Collecting inbox...").await?;

        // 1. Observe: collect from all connectors with dedup
        let mut new_items = Vec::new();
        for query in &ctx.queries {
            let items = query.fetch_inbox().await.unwrap_or_default();
            for item in items {
                // 外部サービス側の完了状態を確認
                let pending = query.is_pending(&item.external_id).await.unwrap_or(true);
                if !pending {
                    continue;
                }

                // Cortex 側の重複チェック
                let exists = ctx
                    .cortex
                    .inbox_exists(&item.source, &item.external_id)
                    .await?;
                if !exists {
                    new_items.push(item);
                }
            }
        }

        if new_items.is_empty() {
            ctx.gateway.send("✅ Inbox is empty!").await?;
            return Ok(());
        }

        ctx.gateway
            .send(&format!("Found {} new items.", new_items.len()))
            .await?;

        // 2. Internalize: create inbox Nodes
        let mut inbox_nodes = Vec::new();
        for item in &new_items {
            let node = ctx
                .cortex
                .insert_node(&NewNode {
                    node_type: NodeType::Inbox,
                    category: Category::Gtd,
                    title: item.title.clone(),
                    content: Some(item.content.clone()),
                    metadata: serde_json::json!({
                        "source": item.source,
                        "external_id": item.external_id,
                        "status": "unprocessed",
                        "captured_at": item.timestamp.to_rfc3339(),
                    }),
                })
                .await?;
            inbox_nodes.push(node);
        }

        // 3. Cross-source dedup: link same-topic items via Edge
        // Compare pairs of new inbox nodes for topic similarity
        if inbox_nodes.len() > 1 {
            for i in 0..inbox_nodes.len() {
                for j in (i + 1)..inbox_nodes.len() {
                    let a = &inbox_nodes[i];
                    let b = &inbox_nodes[j];
                    // Simple heuristic: same title substring or close timestamps
                    let title_overlap = a
                        .title
                        .split_whitespace()
                        .any(|w| w.len() > 3 && b.title.contains(w));
                    if title_overlap {
                        let _ = ctx
                            .cortex
                            .insert_edge(&NewEdge {
                                from_id: a.id.clone(),
                                to_id: b.id.clone(),
                                relation: "same_topic".to_string(),
                                weight: 0.8,
                                context: Some("cross-source dedup".to_string()),
                            })
                            .await;
                    }
                }
            }
        }

        // 4. Brain: classify items
        let items_json = serde_json::to_string_pretty(
            &new_items
                .iter()
                .map(|item| {
                    serde_json::json!({
                        "external_id": item.external_id,
                        "source": item.source,
                        "title": item.title,
                        "content": item.content.chars().take(300).collect::<String>(),
                    })
                })
                .collect::<Vec<_>>(),
        )
        .unwrap_or_default();

        let context = ctx.context_engine.recall("inbox processing").await?;
        let context_summary = context
            .nodes
            .iter()
            .take(5)
            .map(|n| format!("- {}", n.node.title))
            .collect::<Vec<_>>()
            .join("\n");

        let classifications = ctx
            .brain
            .classify_inbox(&items_json, &context_summary)
            .await?;

        // 5. Intervene: present classifications to user
        let mut output = String::from("**Inbox Classification:**\n\n");
        for c in &classifications {
            let icon = match c.classification.as_str() {
                "do_now" => "⚡",
                "task" => "📋",
                "breakdown" => "🔨",
                "reference" => "📚",
                "skip" => "⏭",
                _ => "❓",
            };
            let title = new_items
                .iter()
                .find(|i| i.external_id == c.external_id)
                .map(|i| i.title.as_str())
                .unwrap_or(&c.external_id);
            output.push_str(&format!(
                "{icon} **{title}** → {}\n  _{}_\n\n",
                c.classification, c.reason
            ));
        }

        ctx.gateway.send(&output).await?;

        // 6. Update inbox Node statuses based on classification
        for c in &classifications {
            let status = match c.classification.as_str() {
                "skip" => "skipped",
                _ => "processed",
            };
            if let Some(node) = inbox_nodes.iter().find(|n| {
                n.metadata.get("external_id").and_then(|v| v.as_str()) == Some(&c.external_id)
            }) {
                let mut meta = node.metadata.clone();
                meta["status"] = serde_json::json!(status);
                meta["classification"] = serde_json::json!(c.classification);
                ctx.cortex
                    .update_node(
                        &node.id,
                        &anima_core::NodeUpdate {
                            metadata: Some(meta),
                            ..Default::default()
                        },
                    )
                    .await?;
            }
        }

        ctx.gateway
            .send(&format!("Processed {} inbox items.", classifications.len()))
            .await?;

        Ok(())
    }
}
