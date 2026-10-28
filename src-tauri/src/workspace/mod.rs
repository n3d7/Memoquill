//! Local application services. Native adapters receive no database connection.
mod assets;
mod database;
mod error;
mod faults;
mod migrations;
mod operations;
mod owner;
mod recovery;
mod service;
mod types;

pub use error::{AppError, ErrorCode};
pub use service::ApplicationService;
pub use types::*;

#[cfg(test)]
mod tests;
