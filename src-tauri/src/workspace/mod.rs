//! Local application services. Native adapters receive no database connection.
mod assets;
mod content;
mod database;
mod error;
mod faults;
mod migrations;
mod operations;
mod owner;
mod recovery;
mod search;
mod service;
mod sources;
mod types;

pub use content::*;
pub use error::{AppError, ErrorCode};
pub use search::{SearchHit, SearchPage};
pub use service::ApplicationService;
pub use sources::*;
pub use types::*;

#[cfg(test)]
mod tests;
