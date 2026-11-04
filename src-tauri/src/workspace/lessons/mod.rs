use super::{AppError, ErrorCode, error::Result};
use serde_json::Value;
mod storage;
mod types;
mod validation;
pub use types::*;
use validation::validate_composition;
#[cfg(test)]
mod tests;
fn invalid() -> AppError {
    AppError::new(ErrorCode::InvalidRequest)
}
fn conflict() -> AppError {
    AppError::new(ErrorCode::RevisionConflict)
}
fn corrupt() -> AppError {
    AppError::new(ErrorCode::RecoveryRequired)
}

pub(super) fn registry() -> Vec<RegistryDescriptor> {
    validation::discover()
}
