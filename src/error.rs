pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    Redis(#[from] redis::RedisError),

    #[error(transparent)]
    TokioSend(#[from] tokio::sync::broadcast::error::SendError<()>),

    /// Generic Error to be used Handlers
    #[error(transparent)]
    Generic(#[from] Box<dyn std::error::Error + Send + Sync>),
}
