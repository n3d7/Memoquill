//! Frozen subject/topic checks. Coverage is evidence by outcome, never mastery.
use super::{AppError, ErrorCode, error::Result};
mod storage;
mod types;
pub(super) use storage::{feedback_ready, reserved, resolve_start, validate_native_data};
pub use types::*;
fn invalid() -> AppError {
    AppError::new(ErrorCode::InvalidRequest)
}
fn corrupt() -> AppError {
    AppError::new(ErrorCode::RecoveryRequired)
}
