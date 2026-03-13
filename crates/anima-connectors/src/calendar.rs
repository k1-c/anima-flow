use anima_core::{Error, Result};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use serde::Deserialize;

use crate::traits::{
    ConnectorMeta, ConnectorMutation, ConnectorQuery, InboxItem, MutationRequest, MutationResult,
};

pub struct CalendarConnector {
    http: reqwest::Client,
    api_key: String,
    calendar_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EventListResponse {
    items: Option<Vec<CalendarEvent>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CalendarEvent {
    id: String,
    summary: Option<String>,
    description: Option<String>,
    start: Option<EventTime>,
    end: Option<EventTime>,
    status: Option<String>,
    html_link: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EventTime {
    date_time: Option<String>,
    date: Option<String>,
}

impl CalendarConnector {
    pub fn new(api_key: String, calendar_id: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_key,
            calendar_id,
        }
    }
}

impl ConnectorMeta for CalendarConnector {
    fn source_name(&self) -> &str {
        "calendar"
    }
}

#[async_trait]
impl ConnectorQuery for CalendarConnector {
    async fn fetch_inbox(&self) -> Result<Vec<InboxItem>> {
        let now = Utc::now();
        let time_min = now.to_rfc3339();
        let time_max = (now + Duration::days(2)).to_rfc3339();

        let url = format!(
            "https://www.googleapis.com/calendar/v3/calendars/{}/events",
            self.calendar_id
        );

        let resp = self
            .http
            .get(&url)
            .query(&[
                ("key", self.api_key.as_str()),
                ("timeMin", &time_min),
                ("timeMax", &time_max),
                ("singleEvents", "true"),
                ("orderBy", "startTime"),
                ("maxResults", "20"),
            ])
            .send()
            .await
            .map_err(|e| Error::Connector {
                connector: "calendar".into(),
                message: e.to_string(),
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(Error::Connector {
                connector: "calendar".into(),
                message: format!("{status}: {body}"),
            });
        }

        let events: EventListResponse = resp.json().await.map_err(|e| Error::Connector {
            connector: "calendar".into(),
            message: e.to_string(),
        })?;

        Ok(events
            .items
            .unwrap_or_default()
            .into_iter()
            .filter(|e| e.status.as_deref() != Some("cancelled"))
            .map(|event| {
                let start = event
                    .start
                    .as_ref()
                    .and_then(|s| s.date_time.as_deref().or(s.date.as_deref()))
                    .unwrap_or("unknown");
                let end = event
                    .end
                    .as_ref()
                    .and_then(|s| s.date_time.as_deref().or(s.date.as_deref()))
                    .unwrap_or("unknown");

                InboxItem {
                    source: "calendar".into(),
                    external_id: event.id,
                    title: event.summary.unwrap_or_else(|| "(no title)".into()),
                    content: format!(
                        "{} — {}\n{}",
                        start,
                        end,
                        event.description.as_deref().unwrap_or("")
                    ),
                    metadata: serde_json::json!({
                        "start": start,
                        "end": end,
                        "link": event.html_link,
                    }),
                    timestamp: start.parse().unwrap_or_else(|_| Utc::now()),
                }
            })
            .collect())
    }

    async fn is_pending(&self, _external_id: &str) -> Result<bool> {
        // カレンダーイベントは終了時刻まで常に「pending」
        Ok(true)
    }
}

#[async_trait]
impl ConnectorMutation for CalendarConnector {
    fn supported_actions(&self) -> &[&str] {
        &["create_event"]
    }

    async fn execute(&self, request: &MutationRequest) -> Result<MutationResult> {
        match request.action.as_str() {
            "create_event" => {
                // TODO: Google Calendar API で予定を作成
                Ok(MutationResult {
                    success: false,
                    message: "create_event is not yet implemented".into(),
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
