use anima_core::{Edge, NewEdge, NewNode, Node, NodeType, NodeUpdate, Result, id};
use sqlx::PgPool;

pub struct CortexRepo {
    pool: PgPool,
}

impl CortexRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    // -- Node CRUD --

    pub async fn insert_node(&self, node: &NewNode) -> Result<Node> {
        let id = id::generate();
        let row = sqlx::query_as::<_, Node>(
            r#"
            INSERT INTO nodes (id, type, category, title, content, metadata)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, user_id, type, category, title, content, metadata,
                      access_count, created_at, updated_at
            "#,
        )
        .bind(&id)
        .bind(node.node_type.as_str())
        .bind(node.category.as_str())
        .bind(&node.title)
        .bind(&node.content)
        .bind(&node.metadata)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn get_node(&self, id: &str) -> Result<Option<Node>> {
        let row = sqlx::query_as::<_, Node>(
            r#"
            SELECT id, user_id, type, category, title, content, metadata,
                   access_count, created_at, updated_at
            FROM nodes WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn update_node(&self, id: &str, update: &NodeUpdate) -> Result<Node> {
        let row = sqlx::query_as::<_, Node>(
            r#"
            UPDATE nodes SET
                title = COALESCE($2, title),
                content = COALESCE($3, content),
                metadata = COALESCE($4, metadata),
                updated_at = now()
            WHERE id = $1
            RETURNING id, user_id, type, category, title, content, metadata,
                      access_count, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&update.title)
        .bind(&update.content)
        .bind(&update.metadata)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn delete_node(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM nodes WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn increment_access(&self, id: &str) -> Result<()> {
        sqlx::query("UPDATE nodes SET access_count = access_count + 1 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // -- Edge CRUD --

    pub async fn insert_edge(&self, edge: &NewEdge) -> Result<Edge> {
        let row = sqlx::query_as::<_, Edge>(
            r#"
            INSERT INTO edges (from_id, to_id, relation, weight, context)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING from_id, to_id, relation, weight, context, created_at
            "#,
        )
        .bind(&edge.from_id)
        .bind(&edge.to_id)
        .bind(&edge.relation)
        .bind(edge.weight)
        .bind(&edge.context)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn get_edges_from(&self, node_id: &str) -> Result<Vec<Edge>> {
        let rows = sqlx::query_as::<_, Edge>(
            r#"
            SELECT from_id, to_id, relation, weight, context, created_at
            FROM edges WHERE from_id = $1
            "#,
        )
        .bind(node_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn get_edges_to(&self, node_id: &str) -> Result<Vec<Edge>> {
        let rows = sqlx::query_as::<_, Edge>(
            r#"
            SELECT from_id, to_id, relation, weight, context, created_at
            FROM edges WHERE to_id = $1
            "#,
        )
        .bind(node_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    // -- Inbox dedup --

    pub async fn inbox_exists(&self, source: &str, external_id: &str) -> Result<bool> {
        let row: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM nodes
                WHERE type = 'inbox'
                  AND metadata->>'source' = $1
                  AND metadata->>'external_id' = $2
            )
            "#,
        )
        .bind(source)
        .bind(external_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.0)
    }

    // -- Query --

    pub async fn find_by_type(&self, node_type: NodeType) -> Result<Vec<Node>> {
        let rows = sqlx::query_as::<_, Node>(
            r#"
            SELECT id, user_id, type, category, title, content, metadata,
                   access_count, created_at, updated_at
            FROM nodes WHERE type = $1
            ORDER BY updated_at DESC
            "#,
        )
        .bind(node_type.as_str())
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn find_by_title(&self, query: &str) -> Result<Vec<Node>> {
        let pattern = format!("%{query}%");
        let rows = sqlx::query_as::<_, Node>(
            r#"
            SELECT id, user_id, type, category, title, content, metadata,
                   access_count, created_at, updated_at
            FROM nodes WHERE title ILIKE $1
            ORDER BY updated_at DESC
            "#,
        )
        .bind(&pattern)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn find_by_metadata(&self, key: &str, value: &str) -> Result<Vec<Node>> {
        let rows = sqlx::query_as::<_, Node>(
            r#"
            SELECT id, user_id, type, category, title, content, metadata,
                   access_count, created_at, updated_at
            FROM nodes WHERE metadata->>$1 = $2
            ORDER BY updated_at DESC
            "#,
        )
        .bind(key)
        .bind(value)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn full_text_search(&self, query: &str) -> Result<Vec<Node>> {
        let rows = sqlx::query_as::<_, Node>(
            r#"
            SELECT id, user_id, type, category, title, content, metadata,
                   access_count, created_at, updated_at
            FROM nodes
            WHERE to_tsvector('simple', coalesce(title, '') || ' ' || coalesce(content, ''))
                  @@ plainto_tsquery('simple', $1)
            ORDER BY updated_at DESC
            "#,
        )
        .bind(query)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }
}
