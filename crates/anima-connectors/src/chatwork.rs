use anima_core::{Error, Result};
use async_trait::async_trait;

use crate::traits::{
    ConnectorMeta, ConnectorMutation, ConnectorQuery, InboxItem, MutationRequest, MutationResult,
};

pub struct ChatworkConnector {
    http: reqwest::Client,
    api_token: String,
    room_id: String,
}

impl ChatworkConnector {
    pub fn new(api_token: String, room_id: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_token,
            room_id,
        }
    }
}

impl ConnectorMeta for ChatworkConnector {
    fn source_name(&self) -> &str {
        "chatwork"
    }
}

#[async_trait]
impl ConnectorQuery for ChatworkConnector {
    async fn fetch_inbox(&self) -> Result<Vec<InboxItem>> {
        // TODO: Chatwork REST API でメッセージを取得
        Ok(vec![])
    }

    async fn is_pending(&self, _external_id: &str) -> Result<bool> {
        Ok(true)
    }
}

#[async_trait]
impl ConnectorMutation for ChatworkConnector {
    fn supported_actions(&self) -> &[&str] {
        &["send_message"]
    }

    async fn execute(&self, request: &MutationRequest) -> Result<MutationResult> {
        match request.action.as_str() {
            "send_message" => {
                let body = request.payload["body"].as_str().unwrap_or_default();

                let resp = self
                    .http
                    .post(format!(
                        "https://api.chatwork.com/v2/rooms/{}/messages",
                        self.room_id
                    ))
                    .header("X-ChatWorkToken", &self.api_token)
                    .form(&[("body", body)])
                    .send()
                    .await
                    .map_err(|e| Error::Connector {
                        connector: "chatwork".into(),
                        message: e.to_string(),
                    })?;

                Ok(MutationResult {
                    success: resp.status().is_success(),
                    message: if resp.status().is_success() {
                        "message sent".into()
                    } else {
                        format!("failed: {}", resp.status())
                    },
                    data: None,
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
