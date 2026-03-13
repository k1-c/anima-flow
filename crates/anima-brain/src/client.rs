use anima_core::{Config, Error, Result};
use serde::{Deserialize, Serialize};

pub struct AnthropicClient {
    http: reqwest::Client,
    api_key: String,
    model: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
struct ApiRequest {
    model: String,
    max_tokens: u32,
    system: String,
    messages: Vec<Message>,
}

#[derive(Debug, Deserialize)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    pub text: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse {
    pub content: Vec<ContentBlock>,
    pub usage: Usage,
}

// -- Convenience method return types --

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CueExtraction {
    #[serde(default)]
    pub entities: Vec<String>,
    #[serde(default)]
    pub intent: String,
    #[serde(default)]
    pub time_refs: Vec<String>,
    #[serde(default)]
    pub topics: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InboxClassification {
    pub external_id: String,
    pub classification: String,
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskBreakdown {
    pub goal: String,
    pub actions: Vec<SubTask>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubTask {
    pub title: String,
    pub estimate_min: u32,
    #[serde(default)]
    pub depends_on: Vec<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BriefingSynthesis {
    pub greeting: String,
    pub summary: String,
    pub priorities: Vec<String>,
    pub reminders: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReviewSynthesis {
    pub summary: String,
    pub completed: Vec<String>,
    pub in_progress: Vec<String>,
    pub unstarted: Vec<String>,
    pub tomorrow: Vec<String>,
    pub learnings: Vec<String>,
}

impl AnthropicClient {
    pub fn new(config: &Config) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_key: config.anthropic_api_key.clone(),
            model: config.anthropic_model.clone(),
        }
    }

    pub async fn complete(
        &self,
        system: &str,
        messages: Vec<Message>,
        max_tokens: u32,
    ) -> Result<ApiResponse> {
        let body = ApiRequest {
            model: self.model.clone(),
            max_tokens,
            system: system.to_string(),
            messages,
        };

        let resp = self
            .http
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Anthropic(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_else(|_| "unknown".to_string());
            return Err(Error::Anthropic(format!("{status}: {body}")));
        }

        let api_resp: ApiResponse = resp
            .json()
            .await
            .map_err(|e| Error::Anthropic(e.to_string()))?;

        Ok(api_resp)
    }

    /// Extract the text content from a completion response.
    pub fn extract_text(response: &ApiResponse) -> String {
        response
            .content
            .iter()
            .filter_map(|b| b.text.as_deref())
            .collect::<Vec<_>>()
            .join("")
    }

    // -- Convenience methods --

    /// Extract cues (entities, intent, time refs, topics) from user input.
    pub async fn extract_cues(&self, user_input: &str) -> Result<CueExtraction> {
        let resp = self
            .complete(
                crate::prompt::CUE_EXTRACTION,
                vec![Message {
                    role: "user".into(),
                    content: user_input.into(),
                }],
                512,
            )
            .await?;
        let text = Self::extract_text(&resp);
        Ok(serde_json::from_str(&text).unwrap_or_default())
    }

    /// Classify inbox items according to GTD categories.
    pub async fn classify_inbox(
        &self,
        items_json: &str,
        context: &str,
    ) -> Result<Vec<InboxClassification>> {
        let user_content = format!("## Context\n{context}\n\n## Inbox items\n{items_json}");
        let resp = self
            .complete(
                crate::prompt::INBOX_CLASSIFY,
                vec![Message {
                    role: "user".into(),
                    content: user_content,
                }],
                2048,
            )
            .await?;
        let text = Self::extract_text(&resp);
        Ok(serde_json::from_str(&text).unwrap_or_default())
    }

    /// Break down a task into concrete sub-actions.
    pub async fn breakdown_task(&self, task: &str, context: &str) -> Result<TaskBreakdown> {
        let user_content = format!("## Task\n{task}\n\n## Context\n{context}");
        let resp = self
            .complete(
                crate::prompt::TASK_BREAKDOWN,
                vec![Message {
                    role: "user".into(),
                    content: user_content,
                }],
                2048,
            )
            .await?;
        let text = Self::extract_text(&resp);
        serde_json::from_str(&text).map_err(|e| Error::Anthropic(format!("parse error: {e}")))
    }

    /// Synthesize a morning briefing from collected data.
    pub async fn synthesize_briefing(
        &self,
        inbox_summary: &str,
        context_summary: &str,
    ) -> Result<BriefingSynthesis> {
        let user_content = format!(
            "## Inbox\n{inbox_summary}\n\n## Context (recalled from Cortex)\n{context_summary}"
        );
        let resp = self
            .complete(
                crate::prompt::MORNING_BRIEFING,
                vec![Message {
                    role: "user".into(),
                    content: user_content,
                }],
                2048,
            )
            .await?;
        let text = Self::extract_text(&resp);
        serde_json::from_str(&text).map_err(|e| Error::Anthropic(format!("parse error: {e}")))
    }

    /// Synthesize an end-of-day review.
    pub async fn synthesize_review(
        &self,
        activities: &str,
        context_summary: &str,
    ) -> Result<ReviewSynthesis> {
        let user_content = format!(
            "## Today's activities\n{activities}\n\n## Context (recalled from Cortex)\n{context_summary}"
        );
        let resp = self
            .complete(
                crate::prompt::DAILY_REVIEW,
                vec![Message {
                    role: "user".into(),
                    content: user_content,
                }],
                2048,
            )
            .await?;
        let text = Self::extract_text(&resp);
        serde_json::from_str(&text).map_err(|e| Error::Anthropic(format!("parse error: {e}")))
    }

    /// General-purpose chat: respond to user input with recalled context.
    pub async fn chat(&self, user_input: &str, context: &str) -> Result<String> {
        let system = format!(
            "{}\n\n## Recalled context from Cortex\n{context}",
            crate::prompt::CHAT_SYSTEM
        );
        let resp = self
            .complete(
                &system,
                vec![Message {
                    role: "user".into(),
                    content: user_input.into(),
                }],
                4096,
            )
            .await?;
        Ok(Self::extract_text(&resp))
    }
}
