pub mod cfg;
pub mod error;
pub mod handler;
pub mod store;
pub mod ytdl;

pub type Result<T> = std::result::Result<T, error::Error>;
