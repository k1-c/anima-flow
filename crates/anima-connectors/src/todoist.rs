use anima_core::{Error, Result};
use async_trait::async_trait;
use chrono::Utc;
use serde::Deserialize;

use crate::traits::{
    ConnectorMeta, ConnectorMutation, ConnectorQuery, InboxItem, MutationRequest, MutationResult,
};

pub struct TodoistConnector {
    http: reqwest::Client,
    api_key: String,
}

#[derive(Debug, Deserialize)]
struct TodoistTask {
    id: String,
    content: String,
    description: String,
    is_completed: bool,
    priority: u8,
    due: Option<TodoistDue>,
    created_at: String,
}

#[derive(Debug, Deserialize)]
struct TodoistDue {
    date: String,
    string: String,
}

impl TodoistConnector {
    pub fn new(api_key: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_key,
        }
    }
}

impl ConnectorMeta for TodoistConnector {
    fn source_name(&self) -> &str {
        "todoist"
    }
}

#[async_trait]
impl ConnectorQuery for TodoistConnector {
    async fn fetch_inbox(&self) -> Result<Vec<InboxItem>> {
        let resp = self
            .http
            .get("https://api.todoist.com/rest/v2/tasks")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .query(&[("filter", "today | overdue")])
            .send()
            .await
            .map_err(|e| Error::Connector {
                connector: "todoist".into(),
                message: e.to_string(),
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(Error::Connector {
                connector: "todoist".into(),
                message: format!("{status}: {body}"),
            });
        }

        let tasks: Vec<TodoistTask> = resp.json().await.map_err(|e| Error::Connector {
            connector: "todoist".into(),
            message: e.to_string(),
        })?;

        Ok(tasks
            .into_iter()
            .filter(|t| !t.is_completed)
            .map(|task| {
                let due_info = task
                    .due
                    .as_ref()
                    .map(|d| format!("Due: {} ({})", d.date, d.string))
                    .unwrap_or_else(|| "No due date".into());

                InboxItem {
                    source: "todoist".into(),
                    external_id: task.id.clone(),
                    title: task.content,
                    content: format!(
                        "{}\nPriority: {}\n{}",
                        task.description, task.priority, due_info
                    ),
                    metadata: serde_json::json!({
                        "priority": task.priority,
                        "todoist_id": task.id,
                    }),
                    timestamp: task.created_at.parse().unwrap_or_else(|_| Utc::now()),
                }
            })
            .collect())
    }

    async fn is_pending(&self, external_id: &str) -> Result<bool> {
        let resp = self
            .http
            .get(format!(
                "https://api.todoist.com/rest/v2/tasks/{external_id}"
            ))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| Error::Connector {
                connector: "todoist".into(),
                message: e.to_string(),
            })?;

        if resp.status().is_client_error() {
            // 404 = タスク削除済みまたは完了済み
            return Ok(false);
        }

        let task: TodoistTask = resp.json().await.map_err(|e| Error::Connector {
            connector: "todoist".into(),
            message: e.to_string(),
        })?;

        Ok(!task.is_completed)
    }
}

#[async_trait]
impl ConnectorMutation for TodoistConnector {
    fn supported_actions(&self) -> &[&str] {
        &["complete_task", "create_task"]
    }

    async fn execute(&self, request: &MutationRequest) -> Result<MutationResult> {
        match request.action.as_str() {
            "complete_task" => {
                let task_id = request.payload["task_id"].as_str().unwrap_or_default();

                let resp = self
                    .http
                    .post(format!(
                        "https://api.todoist.com/rest/v2/tasks/{task_id}/close"
                    ))
                    .header("Authorization", format!("Bearer {}", self.api_key))
                    .send()
                    .await
                    .map_err(|e| Error::Connector {
                        connector: "todoist".into(),
                        message: e.to_string(),
                    })?;

                Ok(MutationResult {
                    success: resp.status().is_success(),
                    message: if resp.status().is_success() {
                        "task completed".into()
                    } else {
                        format!("failed: {}", resp.status())
                    },
                    data: None,
                })
            }
            "create_task" => {
                let content = request.payload["content"].as_str().unwrap_or_default();

                let mut body = serde_json::json!({ "content": content });
                if let Some(due) = request.payload.get("due_string") {
                    body["due_string"] = due.clone();
                }

                let resp = self
                    .http
                    .post("https://api.todoist.com/rest/v2/tasks")
                    .header("Authorization", format!("Bearer {}", self.api_key))
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| Error::Connector {
                        connector: "todoist".into(),
                        message: e.to_string(),
                    })?;

                let status = resp.status();
                let resp_body: serde_json::Value =
                    resp.json().await.unwrap_or(serde_json::json!({}));

                Ok(MutationResult {
                    success: status.is_success(),
                    message: if status.is_success() {
                        "task created".into()
                    } else {
                        format!("failed: {status}")
                    },
                    data: Some(resp_body),
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
