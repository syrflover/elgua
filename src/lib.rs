pub mod audio;
pub mod cfg;
pub mod component;
pub mod controller;
pub mod error;
pub mod event;
pub mod handler;
pub mod interaction;
pub mod route;
pub mod store;
pub mod track;
pub mod usecase;

pub type Result<T> = std::result::Result<T, error::Error>;
