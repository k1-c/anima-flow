use anima_core::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Shared types
// ---------------------------------------------------------------------------

/// An item captured from an external service, ready for Inbox processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboxItem {
    pub source: String,
    pub external_id: String,
    pub title: String,
    pub content: String,
    pub metadata: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

/// Identifies a connector service (shared by Query and Mutation).
pub trait ConnectorMeta: Send + Sync {
    /// Unique service name (e.g. "gmail", "slack", "linear").
    fn source_name(&self) -> &str;
}

// ---------------------------------------------------------------------------
// Query — observation / read-only
// ---------------------------------------------------------------------------

/// Read-only access to an external service (observation layer).
///
/// Granting query permission allows Anima to *see* your data in the service
/// but never modify it.
#[async_trait]
pub trait ConnectorQuery: ConnectorMeta {
    /// Fetch unprocessed items from the external service.
    async fn fetch_inbox(&self) -> Result<Vec<InboxItem>>;

    /// Check if an item is still pending (not completed externally).
    async fn is_pending(&self, external_id: &str) -> Result<bool>;
}

// ---------------------------------------------------------------------------
// Mutation — intervention / write
// ---------------------------------------------------------------------------

/// A requested write action against an external service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationRequest {
    /// The action verb (e.g. "send_message", "complete_task", "create_event").
    pub action: String,
    /// Action-specific payload.
    pub payload: serde_json::Value,
}

/// Result of a mutation action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// Write access to an external service (intervention layer).
///
/// Granting mutation permission allows Anima to *act on your behalf*
/// in the service — send messages, complete tasks, create events, etc.
#[async_trait]
pub trait ConnectorMutation: ConnectorMeta {
    /// List the action verbs this connector supports
    /// (e.g. `["send_message", "add_reaction"]`).
    fn supported_actions(&self) -> &[&str];

    /// Execute a write action.
    async fn execute(&self, request: &MutationRequest) -> Result<MutationResult>;
}

// ---------------------------------------------------------------------------
// Backward-compat re-exports (deprecated alias)
// ---------------------------------------------------------------------------

/// Alias for `ConnectorQuery` — kept for migration convenience.
pub trait Connector: ConnectorQuery {}
impl<T: ConnectorQuery> Connector for T {}
