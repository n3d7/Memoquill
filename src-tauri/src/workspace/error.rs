use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    ClockRollback,
    ReviewBudgetExceeded,
    InvalidRequest,
    InvalidTree,
    InvalidReference,
    BudgetExceeded,
    UnsupportedVersion,
    CapabilityUnavailable,
    AlreadyOpen,
    Busy,
    OwnerUnavailable,
    UnsupportedSchema,
    RevisionConflict,
    IdempotencyConflict,
    StorageFailure,
    StorageFull,
    PermissionDenied,
    RecoveryRequired,
    OutcomeUnconfirmed,
}

/// Safe wire error: never includes raw SQL, source data or machine paths.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppError {
    pub code: ErrorCode,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<super::LessonDiagnostic>,
}

pub(super) type Result<T> = std::result::Result<T, AppError>;
impl std::error::Error for AppError {}

impl AppError {
    pub(crate) const fn new(code: ErrorCode) -> Self {
        Self {
            code,
            diagnostics: Vec::new(),
        }
    }
}

impl AppError {
    pub(super) fn with_diagnostics(mut self, diagnostics: Vec<super::LessonDiagnostic>) -> Self {
        self.diagnostics = diagnostics.into_iter().take(16).collect();
        self
    }
}
impl std::fmt::Display for AppError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{:?}", self.code)
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self::new(if error.kind() == std::io::ErrorKind::StorageFull {
            ErrorCode::StorageFull
        } else {
            ErrorCode::StorageFailure
        })
    }
}

impl From<rustix::io::Errno> for AppError {
    fn from(error: rustix::io::Errno) -> Self {
        std::io::Error::from_raw_os_error(error.raw_os_error()).into()
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(error: rusqlite::Error) -> Self {
        use rusqlite::ErrorCode as Sql;
        let code = match error.sqlite_error_code() {
            Some(Sql::DatabaseBusy | Sql::DatabaseLocked) => ErrorCode::Busy,
            Some(Sql::DiskFull) => ErrorCode::StorageFull,
            Some(Sql::DatabaseCorrupt | Sql::NotADatabase) => ErrorCode::RecoveryRequired,
            _ => ErrorCode::StorageFailure,
        };
        Self::new(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn filesystem_full_errors_remain_actionable_without_exposing_native_details() {
        assert_eq!(
            AppError::from(std::io::Error::from_raw_os_error(28)).code,
            ErrorCode::StorageFull
        );
        assert_eq!(
            AppError::from(rustix::io::Errno::NOSPC).code,
            ErrorCode::StorageFull
        );
        let wire = serde_json::to_string(&AppError::from(std::io::Error::other("/private/source")))
            .unwrap();
        assert_eq!(wire, "{\"code\":\"storage_failure\"}");
    }
}
