use crate::Error;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub anthropic_api_key: String,
    pub anthropic_model: String,
    pub embedding_dimensions: usize,

    // Google Calendar
    pub google_calendar_api_key: Option<String>,
    pub google_calendar_id: Option<String>,

    // Gmail (将来用)
    pub google_credentials_json: Option<String>,

    // Slack
    pub slack_bot_token: Option<String>,

    // Linear
    pub linear_api_key: Option<String>,

    // Todoist
    pub todoist_api_key: Option<String>,

    // Chatwork
    pub chatwork_api_token: Option<String>,
    pub chatwork_room_id: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, Error> {
        Ok(Self {
            database_url: env_required("DATABASE_URL")?,
            anthropic_api_key: env_required("ANTHROPIC_API_KEY")?,
            anthropic_model: std::env::var("ANTHROPIC_MODEL")
                .unwrap_or_else(|_| "claude-sonnet-4-20250514".to_string()),
            embedding_dimensions: std::env::var("EMBEDDING_DIMENSIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1536),

            google_calendar_api_key: std::env::var("GOOGLE_CALENDAR_API_KEY").ok(),
            google_calendar_id: std::env::var("GOOGLE_CALENDAR_ID").ok(),
            google_credentials_json: std::env::var("GOOGLE_CREDENTIALS_JSON").ok(),
            slack_bot_token: std::env::var("SLACK_BOT_TOKEN").ok(),
            linear_api_key: std::env::var("LINEAR_API_KEY").ok(),
            todoist_api_key: std::env::var("TODOIST_API_KEY").ok(),
            chatwork_api_token: std::env::var("CHATWORK_API_TOKEN").ok(),
            chatwork_room_id: std::env::var("CHATWORK_ROOM_ID").ok(),
        })
    }
}

fn env_required(key: &str) -> Result<String, Error> {
    std::env::var(key).map_err(|_| Error::Config(format!("{key} is not set")))
}
