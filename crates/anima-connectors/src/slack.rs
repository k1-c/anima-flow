use anima_core::{Error, Result};
use async_trait::async_trait;
use chrono::Utc;
use serde::Deserialize;

use crate::traits::{
    ConnectorMeta, ConnectorMutation, ConnectorQuery, InboxItem, MutationRequest, MutationResult,
};

pub struct SlackConnector {
    http: reqwest::Client,
    bot_token: String,
}

#[derive(Debug, Deserialize)]
struct ConversationsResponse {
    #[allow(dead_code)]
    ok: bool,
    messages: Option<Vec<SlackMessage>>,
    #[allow(dead_code)]
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SlackMessage {
    ts: String,
    text: String,
    user: Option<String>,
    #[allow(dead_code)]
    channel: Option<String>,
    #[allow(dead_code)]
    #[serde(rename = "type")]
    msg_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AuthTestResponse {
    ok: bool,
    user_id: Option<String>,
}

impl SlackConnector {
    pub fn new(bot_token: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            bot_token,
        }
    }

    #[allow(dead_code)]
    async fn get_bot_user_id(&self) -> Result<String> {
        let resp: AuthTestResponse = self
            .http
            .get("https://slack.com/api/auth.test")
            .header("Authorization", format!("Bearer {}", self.bot_token))
            .send()
            .await
            .map_err(|e| Error::Connector {
                connector: "slack".into(),
                message: e.to_string(),
            })?
            .json()
            .await
            .map_err(|e| Error::Connector {
                connector: "slack".into(),
                message: e.to_string(),
            })?;

        resp.user_id.ok_or_else(|| Error::Connector {
            connector: "slack".into(),
            message: "could not determine bot user id".into(),
        })
    }

    /// 最近のDMを取得する。
    async fn fetch_im_messages(&self) -> Result<Vec<InboxItem>> {
        let resp = self
            .http
            .get("https://slack.com/api/conversations.list")
            .header("Authorization", format!("Bearer {}", self.bot_token))
            .query(&[("types", "im"), ("limit", "20")])
            .send()
            .await
            .map_err(|e| Error::Connector {
                connector: "slack".into(),
                message: e.to_string(),
            })?;

        #[derive(Deserialize)]
        struct ChannelListResp {
            #[allow(dead_code)]
            ok: bool,
            channels: Option<Vec<SlackChannel>>,
        }
        #[derive(Deserialize)]
        struct SlackChannel {
            id: String,
        }

        let channel_list: ChannelListResp = resp.json().await.map_err(|e| Error::Connector {
            connector: "slack".into(),
            message: e.to_string(),
        })?;

        let channels = channel_list.channels.unwrap_or_default();
        let mut items = Vec::new();

        for channel in channels.iter().take(10) {
            let resp = self
                .http
                .get("https://slack.com/api/conversations.history")
                .header("Authorization", format!("Bearer {}", self.bot_token))
                .query(&[("channel", channel.id.as_str()), ("limit", "5")])
                .send()
                .await
                .map_err(|e| Error::Connector {
                    connector: "slack".into(),
                    message: e.to_string(),
                })?;

            let history: ConversationsResponse =
                resp.json().await.map_err(|e| Error::Connector {
                    connector: "slack".into(),
                    message: e.to_string(),
                })?;

            if let Some(messages) = history.messages {
                for msg in messages {
                    items.push(InboxItem {
                        source: "slack".into(),
                        external_id: format!("{}:{}", channel.id, msg.ts),
                        title: msg.text.chars().take(80).collect::<String>(),
                        content: msg.text,
                        metadata: serde_json::json!({
                            "channel": channel.id,
                            "ts": msg.ts,
                            "user": msg.user,
                        }),
                        timestamp: parse_slack_ts(&msg.ts),
                    });
                }
            }
        }

        Ok(items)
    }
}

fn parse_slack_ts(ts: &str) -> chrono::DateTime<Utc> {
    ts.split('.')
        .next()
        .and_then(|s| s.parse::<i64>().ok())
        .and_then(|secs| chrono::DateTime::from_timestamp(secs, 0))
        .unwrap_or_else(Utc::now)
}

impl ConnectorMeta for SlackConnector {
    fn source_name(&self) -> &str {
        "slack"
    }
}

#[async_trait]
impl ConnectorQuery for SlackConnector {
    async fn fetch_inbox(&self) -> Result<Vec<InboxItem>> {
        self.fetch_im_messages().await
    }

    async fn is_pending(&self, _external_id: &str) -> Result<bool> {
        // Slack メッセージには組み込みの「完了」状態がない。
        // 完了管理は Cortex 側で追跡する。
        Ok(true)
    }
}

#[async_trait]
impl ConnectorMutation for SlackConnector {
    fn supported_actions(&self) -> &[&str] {
        &["send_message", "add_reaction"]
    }

    async fn execute(&self, request: &MutationRequest) -> Result<MutationResult> {
        match request.action.as_str() {
            "send_message" => {
                let channel = request.payload["channel"].as_str().unwrap_or_default();
                let text = request.payload["text"].as_str().unwrap_or_default();

                let resp = self
                    .http
                    .post("https://slack.com/api/chat.postMessage")
                    .header("Authorization", format!("Bearer {}", self.bot_token))
                    .json(&serde_json::json!({
                        "channel": channel,
                        "text": text,
                    }))
                    .send()
                    .await
                    .map_err(|e| Error::Connector {
                        connector: "slack".into(),
                        message: e.to_string(),
                    })?;

                let body: serde_json::Value = resp.json().await.map_err(|e| Error::Connector {
                    connector: "slack".into(),
                    message: e.to_string(),
                })?;

                let ok = body["ok"].as_bool().unwrap_or(false);
                Ok(MutationResult {
                    success: ok,
                    message: if ok {
                        "message sent".into()
                    } else {
                        body["error"]
                            .as_str()
                            .unwrap_or("unknown error")
                            .to_string()
                    },
                    data: Some(body),
                })
            }
            "add_reaction" => {
                // TODO: reactions.add API
                Ok(MutationResult {
                    success: false,
                    message: "add_reaction is not yet implemented".into(),
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
