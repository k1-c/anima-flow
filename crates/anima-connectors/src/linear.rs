use anima_core::{Error, Result};
use async_trait::async_trait;
use chrono::Utc;
use serde::Deserialize;

use crate::traits::{
    ConnectorMeta, ConnectorMutation, ConnectorQuery, InboxItem, MutationRequest, MutationResult,
};

pub struct LinearConnector {
    http: reqwest::Client,
    api_key: String,
}

#[derive(Debug, Deserialize)]
struct GqlResponse {
    data: Option<GqlData>,
}

#[derive(Debug, Deserialize)]
struct GqlData {
    issues: GqlIssues,
}

#[derive(Debug, Deserialize)]
struct GqlIssues {
    nodes: Vec<GqlIssue>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GqlIssue {
    id: String,
    identifier: String,
    title: String,
    description: Option<String>,
    priority: f64,
    state: GqlState,
    due_date: Option<String>,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
struct GqlState {
    name: String,
    #[serde(rename = "type")]
    state_type: String,
}

impl LinearConnector {
    pub fn new(api_key: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_key,
        }
    }

    async fn query_assigned_issues(&self) -> Result<Vec<GqlIssue>> {
        let query = r#"
            query {
                issues(
                    filter: {
                        assignee: { isMe: { eq: true } }
                        state: { type: { nin: ["completed", "canceled"] } }
                    }
                    orderBy: updatedAt
                    first: 50
                ) {
                    nodes {
                        id
                        identifier
                        title
                        description
                        priority
                        state { name type }
                        dueDate
                        updatedAt
                    }
                }
            }
        "#;

        let resp = self
            .http
            .post("https://api.linear.app/graphql")
            .header("Authorization", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({ "query": query }))
            .send()
            .await
            .map_err(|e| Error::Connector {
                connector: "linear".into(),
                message: e.to_string(),
            })?;

        let body: GqlResponse = resp.json().await.map_err(|e| Error::Connector {
            connector: "linear".into(),
            message: e.to_string(),
        })?;

        Ok(body.data.map(|d| d.issues.nodes).unwrap_or_default())
    }

    async fn graphql_mutation(&self, query: &str) -> Result<serde_json::Value> {
        let resp = self
            .http
            .post("https://api.linear.app/graphql")
            .header("Authorization", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({ "query": query }))
            .send()
            .await
            .map_err(|e| Error::Connector {
                connector: "linear".into(),
                message: e.to_string(),
            })?;

        resp.json().await.map_err(|e| Error::Connector {
            connector: "linear".into(),
            message: e.to_string(),
        })
    }
}

impl ConnectorMeta for LinearConnector {
    fn source_name(&self) -> &str {
        "linear"
    }
}

#[async_trait]
impl ConnectorQuery for LinearConnector {
    async fn fetch_inbox(&self) -> Result<Vec<InboxItem>> {
        let issues = self.query_assigned_issues().await?;

        Ok(issues
            .into_iter()
            .map(|issue| {
                let content = format!(
                    "[{}] {} — {}\nPriority: {}{}",
                    issue.state.name,
                    issue.identifier,
                    issue.description.as_deref().unwrap_or("(no description)"),
                    issue.priority,
                    issue
                        .due_date
                        .as_deref()
                        .map(|d| format!("\nDue: {d}"))
                        .unwrap_or_default(),
                );

                InboxItem {
                    source: "linear".into(),
                    external_id: issue.identifier,
                    title: issue.title,
                    content,
                    metadata: serde_json::json!({
                        "state": issue.state.name,
                        "state_type": issue.state.type_field(),
                        "priority": issue.priority,
                        "linear_id": issue.id,
                    }),
                    timestamp: issue.updated_at.parse().unwrap_or_else(|_| Utc::now()),
                }
            })
            .collect())
    }

    async fn is_pending(&self, _external_id: &str) -> Result<bool> {
        // クエリ自体が未完了のみをフィルタしている
        Ok(true)
    }
}

#[async_trait]
impl ConnectorMutation for LinearConnector {
    fn supported_actions(&self) -> &[&str] {
        &["update_issue_state", "create_comment"]
    }

    async fn execute(&self, request: &MutationRequest) -> Result<MutationResult> {
        match request.action.as_str() {
            "update_issue_state" => {
                let issue_id = request.payload["issue_id"].as_str().unwrap_or_default();
                let state_id = request.payload["state_id"].as_str().unwrap_or_default();

                let query = format!(
                    r#"mutation {{ issueUpdate(id: "{issue_id}", input: {{ stateId: "{state_id}" }}) {{ success }} }}"#
                );
                let body = self.graphql_mutation(&query).await?;
                let success = body["data"]["issueUpdate"]["success"]
                    .as_bool()
                    .unwrap_or(false);

                Ok(MutationResult {
                    success,
                    message: if success {
                        "issue state updated".into()
                    } else {
                        "failed to update issue state".into()
                    },
                    data: Some(body),
                })
            }
            "create_comment" => {
                let issue_id = request.payload["issue_id"].as_str().unwrap_or_default();
                let body_text = request.payload["body"].as_str().unwrap_or_default();

                let query = format!(
                    r#"mutation {{ commentCreate(input: {{ issueId: "{issue_id}", body: "{body_text}" }}) {{ success }} }}"#
                );
                let body = self.graphql_mutation(&query).await?;
                let success = body["data"]["commentCreate"]["success"]
                    .as_bool()
                    .unwrap_or(false);

                Ok(MutationResult {
                    success,
                    message: if success {
                        "comment created".into()
                    } else {
                        "failed to create comment".into()
                    },
                    data: Some(body),
                })
            }
            _ => Ok(MutationResult {
                success: false,
                message: format!("unsupported action: {}", request.action),
                data: None,
            }),
        }
    }
}

impl GqlState {
    fn type_field(&self) -> &str {
        &self.state_type
    }
}
