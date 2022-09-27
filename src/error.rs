use serde::Deserialize;
use songbird::{error::JoinError, input, tracks};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("songbird::error::JoinError {0}")]
    JoinError(#[from] JoinError),

    #[error("songbird::input::error::Error {0}")]
    InputError(#[from] input::error::Error),

    #[error("songbird::tracks::TrackError {0}")]
    TrackError(#[from] tracks::TrackError),

    #[error("sqlx::Error {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("toshi::ToshiClientError {0}")]
    ToshiClientError(#[from] toshi::ToshiClientError),

    #[error("ToshiError {0}")]
    ToshiError(#[from] ToshiError),

    #[error("serenity::Error {0}")]
    SerenityError(#[from] serenity::Error),

    #[error("reqwest: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("error: {0}")]
    CustomError(String),
}

#[derive(Debug, Deserialize)]
pub struct ToshiError {
    #[serde(default)]
    pub status: u16,
    pub message: String,
}

impl std::error::Error for ToshiError {}

impl std::fmt::Display for ToshiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "status = {}; message = {}", self.status, self.message)
    }
}

impl Default for ToshiError {
    fn default() -> Self {
        Self {
            status: 0,
            message: "unknown error".to_string(),
        }
    }
}
