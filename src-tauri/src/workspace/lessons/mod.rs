use super::{AppError, ErrorCode, error::Result};
use serde_json::Value;
pub(crate) mod admission;
mod catalog;
mod diagnostics;
mod storage;
mod types;
mod validation;
pub(super) use storage::validate_native_data;
pub use types::*;
#[cfg(test)]
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
