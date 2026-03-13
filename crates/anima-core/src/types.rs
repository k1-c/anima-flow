use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// -- NodeType --

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    Person,
    Space,
    Episode,
    Decision,
    Daily,
    Procedure,
    Domain,
    Learning,
    Inbox,
    Preference,
    Pattern,
}

impl NodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Person => "person",
            Self::Space => "space",
            Self::Episode => "episode",
            Self::Decision => "decision",
            Self::Daily => "daily",
            Self::Procedure => "procedure",
            Self::Domain => "domain",
            Self::Learning => "learning",
            Self::Inbox => "inbox",
            Self::Preference => "preference",
            Self::Pattern => "pattern",
        }
    }
}

// -- Category --

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Memory,
    Knowledge,
    Ssot,
    Gtd,
}

impl Category {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Memory => "memory",
            Self::Knowledge => "knowledge",
            Self::Ssot => "ssot",
            Self::Gtd => "gtd",
        }
    }
}

// -- Node --

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Node {
    pub id: String,
    pub user_id: Option<Uuid>,
    #[sqlx(rename = "type")]
    pub node_type: String,
    pub category: String,
    pub title: String,
    pub content: Option<String>,
    pub metadata: serde_json::Value,
    // embedding is excluded from default queries (large vector)
    pub access_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewNode {
    pub node_type: NodeType,
    pub category: Category,
    pub title: String,
    pub content: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Default)]
pub struct NodeUpdate {
    pub title: Option<String>,
    pub content: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

// -- Edge --

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Edge {
    pub from_id: String,
    pub to_id: String,
    pub relation: String,
    pub weight: f32,
    pub context: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewEdge {
    pub from_id: String,
    pub to_id: String,
    pub relation: String,
    pub weight: f32,
    pub context: Option<String>,
}
