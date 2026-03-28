#![deny(clippy::all, clippy::pedantic, warnings)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::must_use_candidate)]

pub mod bookmarks;
pub mod history;
pub mod model;
pub mod search_engine;

#[cfg(feature = "iced-gui")]
pub mod app;

pub use search_engine::{BrowserConfig, SearchEngine, SearchEngineRegistry};
