use anima_core::{Node, Result};
use sqlx::PgPool;

/// Node with its activation score from spreading activation.
#[derive(Debug, Clone)]
pub struct ActivatedNode {
    pub node: Node,
    pub score: f64,
}

#[derive(Debug, sqlx::FromRow)]
struct ActivatedRow {
    id: String,
    user_id: Option<uuid::Uuid>,
    #[sqlx(rename = "type")]
    node_type: String,
    category: String,
    title: String,
    content: Option<String>,
    metadata: serde_json::Value,
    access_count: i32,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    activation_score: f64,
}

/// Run spreading activation from seed nodes through the Edge graph.
///
/// Uses a recursive CTE to traverse edges with exponential decay.
pub async fn spreading_activation(
    pool: &PgPool,
    seed_ids: &[String],
    max_depth: i32,
    decay: f32,
    limit: i64,
) -> Result<Vec<ActivatedNode>> {
    let rows = sqlx::query_as::<_, ActivatedRow>(
        r#"
        WITH RECURSIVE activated AS (
            SELECT e.to_id AS node_id,
                   (e.weight * $2::real)::float8 AS score,
                   1 AS depth
            FROM edges e
            WHERE e.from_id = ANY($1)

            UNION ALL

            SELECT e.to_id,
                   (a.score * e.weight * $2::real)::float8,
                   a.depth + 1
            FROM activated a
            JOIN edges e ON e.from_id = a.node_id
            WHERE a.depth < $3
              AND a.score > 0.01
        )
        SELECT n.id, n.user_id, n.type, n.category, n.title, n.content,
               n.metadata, n.access_count, n.created_at, n.updated_at,
               MAX(a.score) AS activation_score
        FROM activated a
        JOIN nodes n ON n.id = a.node_id
        WHERE n.id != ALL($1)
        GROUP BY n.id, n.user_id, n.type, n.category, n.title, n.content,
                 n.metadata, n.access_count, n.created_at, n.updated_at
        ORDER BY activation_score DESC
        LIMIT $4
        "#,
    )
    .bind(seed_ids)
    .bind(decay)
    .bind(max_depth)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let result = rows
        .into_iter()
        .map(|r| ActivatedNode {
            node: Node {
                id: r.id,
                user_id: r.user_id,
                node_type: r.node_type,
                category: r.category,
                title: r.title,
                content: r.content,
                metadata: r.metadata,
                access_count: r.access_count,
                created_at: r.created_at,
                updated_at: r.updated_at,
            },
            score: r.activation_score,
        })
        .collect();

    Ok(result)
}
