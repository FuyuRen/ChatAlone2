use axum::extract::ws::Message;
use tokio::sync::mpsc;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("heartbeat timeout")]
    HeartBeatTimeout,

    #[error("send error: {0}")]
    SendError(#[from] mpsc::error::SendError<Message>),

    #[error("{0}")]
    Custom(String),
}