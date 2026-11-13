//! Local application services. Native adapters receive no database connection.
mod assets;
mod content;
mod courses;
mod database;
mod error;
mod faults;
mod lessons;
mod migrations;
mod operations;
mod owner;
mod packages;
mod recovery;
mod search;
mod service;
mod sources;
mod types;

pub use content::*;
pub use courses::*;
pub use error::{AppError, ErrorCode};
pub use lessons::*;
pub use packages::*;
pub use search::{SearchHit, SearchPage};
pub use service::ApplicationService;
pub use sources::*;
pub use types::*;

#[cfg(test)]
mod tests;

pub(crate) use lessons::admission as lesson_admission;
