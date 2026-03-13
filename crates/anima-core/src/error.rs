pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("database: {0}")]
    Db(#[from] sqlx::Error),

    #[error("anthropic api: {0}")]
    Anthropic(String),

    #[error("connector({connector}): {message}")]
    Connector { connector: String, message: String },

    #[error("context engine: {0}")]
    Context(String),

    #[error("config: {0}")]
    Config(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
