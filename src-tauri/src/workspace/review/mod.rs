//! Recall scheduling is a review policy, never an estimate of general competence.
use super::{AppError, ErrorCode, error::Result};
mod algorithm;
mod storage;
mod types;
pub(super) use storage::{eligible, record_review, validate_native_data};
pub use types::*;
#[cfg(test)]
mod tests;

fn invalid() -> AppError {
    AppError::new(ErrorCode::InvalidRequest)
}
fn corrupt() -> AppError {
    AppError::new(ErrorCode::RecoveryRequired)
}
