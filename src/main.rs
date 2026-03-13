use std::sync::Arc;

use anima_brain::AnthropicClient;
use anima_connectors::{ConnectorMutation, ConnectorQuery};
use anima_context::ContextPipeline;
use anima_core::Config;
use anima_cortex::CortexRepo;
use anima_gateway::{CliGateway, Gateway};
use anima_skills::{Skill, SkillContext};
use clap::{Parser, Subcommand};

type Connectors = (
    Vec<Arc<dyn ConnectorQuery>>,
    Vec<Arc<dyn ConnectorMutation>>,
);
use sqlx::postgres::PgPool;

#[derive(Parser)]
#[command(name = "anima-flow", about = "Your second cortex, always on.")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Morning briefing
    Morning,
    /// Process inbox
    Inbox,
    /// Break down a task
    Breakdown {
        /// Task description
        task: String,
    },
    /// End-of-day review
    Review,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    dotenvy::dotenv().ok();

    let config = Config::from_env()?;
    let pool = PgPool::connect(&config.database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    let cortex = Arc::new(CortexRepo::new(pool));
    let brain = Arc::new(AnthropicClient::new(&config));
    let context_engine = Arc::new(ContextPipeline::new(cortex.clone(), brain.clone()));
    let gateway: Arc<dyn Gateway> = Arc::new(CliGateway::new());

    // --- コネクタ組み立て（env vars が設定されたサービスのみ有効化）---
    let (queries, mutations) = build_connectors(&config);

    let enabled_queries: Vec<_> = queries.iter().map(|q| q.source_name()).collect();
    let enabled_mutations: Vec<_> = mutations.iter().map(|m| m.source_name()).collect();
    tracing::info!(
        ?enabled_queries,
        ?enabled_mutations,
        "connectors initialized"
    );

    let skill_ctx = Arc::new(SkillContext {
        cortex,
        brain,
        context_engine,
        gateway: gateway.clone(),
        queries,
        mutations,
    });

    let cli = Cli::parse();

    match cli.command {
        Some(Command::Morning) => {
            anima_skills::morning::MorningSkill
                .execute(&skill_ctx, "")
                .await?;
        }
        Some(Command::Inbox) => {
            anima_skills::inbox::InboxSkill
                .execute(&skill_ctx, "")
                .await?;
        }
        Some(Command::Breakdown { task }) => {
            anima_skills::breakdown::BreakdownSkill
                .execute(&skill_ctx, &task)
                .await?;
        }
        Some(Command::Review) => {
            anima_skills::review::ReviewSkill
                .execute(&skill_ctx, "")
                .await?;
        }
        None => {
            interactive_loop(&skill_ctx).await?;
        }
    }

    Ok(())
}

/// Config の env vars に基づいてコネクタをインスタンス化する。
/// 認証情報がないサービスはスキップされる。
fn build_connectors(config: &Config) -> Connectors {
    let mut queries: Vec<Arc<dyn ConnectorQuery>> = Vec::new();
    let mut mutations: Vec<Arc<dyn ConnectorMutation>> = Vec::new();

    // Google Calendar — api_key + calendar_id の両方が必要
    if let (Some(api_key), Some(calendar_id)) =
        (&config.google_calendar_api_key, &config.google_calendar_id)
    {
        let c = Arc::new(anima_connectors::calendar::CalendarConnector::new(
            api_key.clone(),
            calendar_id.clone(),
        ));
        queries.push(c.clone());
        mutations.push(c);
    }

    // Slack — bot_token が必要
    if let Some(bot_token) = &config.slack_bot_token {
        let c = Arc::new(anima_connectors::slack::SlackConnector::new(
            bot_token.clone(),
        ));
        queries.push(c.clone());
        mutations.push(c);
    }

    // Linear — api_key が必要
    if let Some(api_key) = &config.linear_api_key {
        let c = Arc::new(anima_connectors::linear::LinearConnector::new(
            api_key.clone(),
        ));
        queries.push(c.clone());
        mutations.push(c);
    }

    // Todoist — api_key が必要
    if let Some(api_key) = &config.todoist_api_key {
        let c = Arc::new(anima_connectors::todoist::TodoistConnector::new(
            api_key.clone(),
        ));
        queries.push(c.clone());
        mutations.push(c);
    }

    // Chatwork — api_token + room_id の両方が必要
    if let (Some(api_token), Some(room_id)) = (&config.chatwork_api_token, &config.chatwork_room_id)
    {
        let c = Arc::new(anima_connectors::chatwork::ChatworkConnector::new(
            api_token.clone(),
            room_id.clone(),
        ));
        queries.push(c.clone());
        mutations.push(c);
    }

    // Gmail — Query のみ（認証情報の仕組みが未実装のため、スタブとして常に追加）
    // TODO: Google OAuth 実装後に credentials_json で条件分岐
    queries.push(Arc::new(anima_connectors::gmail::GmailConnector));

    (queries, mutations)
}

async fn interactive_loop(ctx: &SkillContext) -> anyhow::Result<()> {
    ctx.gateway
        .send(
            "Anima Flow — Your second cortex, always on.\nType /help for commands, or just talk.\n",
        )
        .await?;

    loop {
        ctx.gateway.send("> ").await?;
        let input = ctx.gateway.receive().await?;

        if input.is_empty() {
            continue;
        }

        match input.as_str() {
            "/quit" | "/exit" => break,
            "/help" => {
                ctx.gateway
                    .send(
                        "Commands:\n  /morning    Morning briefing\n  /inbox      Process inbox\n  /breakdown  Break down a task\n  /review     Daily review\n  /quit       Exit",
                    )
                    .await?;
            }
            "/morning" => {
                anima_skills::morning::MorningSkill.execute(ctx, "").await?;
            }
            "/inbox" => {
                anima_skills::inbox::InboxSkill.execute(ctx, "").await?;
            }
            "/review" => {
                anima_skills::review::ReviewSkill.execute(ctx, "").await?;
            }
            s if s.starts_with("/breakdown ") => {
                let task = &s["/breakdown ".len()..];
                anima_skills::breakdown::BreakdownSkill
                    .execute(ctx, task)
                    .await?;
            }
            _ => {
                // Free-form: recall context and chat via Brain
                let context = ctx.context_engine.recall(&input).await?;
                let context_str = context
                    .nodes
                    .iter()
                    .take(10)
                    .map(|n| {
                        format!(
                            "[{}] {}: {}",
                            n.node.node_type,
                            n.node.title,
                            n.node
                                .content
                                .as_deref()
                                .unwrap_or("")
                                .chars()
                                .take(300)
                                .collect::<String>()
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                let response = ctx.brain.chat(&input, &context_str).await?;
                ctx.gateway.send(&response).await?;
            }
        }
    }

    Ok(())
}
