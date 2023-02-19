pub mod audio;
pub mod cfg;
pub mod component;
pub mod error;
pub mod event;
pub mod handler;
pub mod store;
pub mod ytdl;

pub type Result<T> = std::result::Result<T, error::Error>;
