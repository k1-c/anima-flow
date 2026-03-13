use anima_core::Result;
use async_trait::async_trait;

use crate::traits::{Skill, SkillContext};

pub struct BreakdownSkill;

#[async_trait]
impl Skill for BreakdownSkill {
    fn name(&self) -> &str {
        "breakdown"
    }

    fn description(&self) -> &str {
        "Break down a large/ambiguous task into concrete actions"
    }

    async fn execute(&self, ctx: &SkillContext, args: &str) -> Result<()> {
        if args.is_empty() {
            ctx.gateway
                .send("Usage: /breakdown <task description>")
                .await?;
            return Ok(());
        }

        ctx.gateway
            .send(&format!("🔨 Breaking down: {args}"))
            .await?;

        // 1. Internalize: recall related context
        let context = ctx.context_engine.recall(args).await?;
        let context_summary = context
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

        // 2. Brain: break down the task
        let breakdown = ctx.brain.breakdown_task(args, &context_summary).await?;

        // 3. Intervene: present subtasks to user
        let mut output = format!("**Goal:** {}\n\n**Actions:**\n\n", breakdown.goal);
        for (i, action) in breakdown.actions.iter().enumerate() {
            let deps = if action.depends_on.is_empty() {
                String::new()
            } else {
                let dep_list: Vec<String> = action
                    .depends_on
                    .iter()
                    .map(|d| format!("#{}", d + 1))
                    .collect();
                format!(" (after {})", dep_list.join(", "))
            };
            output.push_str(&format!(
                "  {}. {} (~{}min){}\n",
                i + 1,
                action.title,
                action.estimate_min,
                deps
            ));
        }

        let total: u32 = breakdown.actions.iter().map(|a| a.estimate_min).sum();
        output.push_str(&format!("\n**Total estimate:** ~{total}min"));

        ctx.gateway.send(&output).await?;

        Ok(())
    }
}
