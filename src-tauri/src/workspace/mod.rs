//! Local application services. Native adapters receive no database connection.
mod database;
mod error;
mod migrations;
mod owner;
mod types;

pub use error::{AppError, ErrorCode};
pub use types::*;

#[cfg(test)]
mod tests;
