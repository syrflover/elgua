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

    #[error("serenity::Error {0}")]
    SerenityError(#[from] serenity::Error),

    #[error("reqwest: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("{0}")]
    CustomError(String),
}
