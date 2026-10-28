use super::{AppError, ErrorCode, error::Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub(super) const MAX_REVISION: i64 = 9_007_199_254_740_991;

macro_rules! identity {
    ($name:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(String);
        impl $name {
            pub fn generate() -> Self {
                Self(Uuid::new_v4().to_string())
            }
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
        impl TryFrom<String> for $name {
            type Error = AppError;
            fn try_from(value: String) -> Result<Self> {
                validate_uuid(&value)?;
                Ok(Self(value))
            }
        }
        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}
identity!(WorkspaceId);
identity!(OperationId);

fn validate_uuid(value: &str) -> Result<()> {
    if Uuid::parse_str(value).is_ok_and(|id| id.get_version_num() == 4 && id.to_string() == value) {
        return Ok(());
    }
    Err(AppError::new(ErrorCode::InvalidRequest))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    System,
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u16", into = "u16")]
pub struct TextScale(u16);
impl TextScale {
    pub fn get(self) -> u16 {
        self.0
    }
}
impl TryFrom<u16> for TextScale {
    type Error = AppError;
    fn try_from(value: u16) -> Result<Self> {
        match value {
            100 | 125 | 150 | 200 => Ok(Self(value)),
            _ => Err(AppError::new(ErrorCode::InvalidRequest)),
        }
    }
}
impl From<TextScale> for u16 {
    fn from(value: TextScale) -> Self {
        value.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReadingPreferences {
    pub theme: Theme,
    pub text_scale: TextScale,
}
impl Default for ReadingPreferences {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            text_scale: TextScale(100),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PreferencesSnapshot {
    pub workspace_id: WorkspaceId,
    pub revision: i64,
    pub preferences: ReadingPreferences,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SavePreferences {
    pub contract_version: u32,
    pub workspace_id: WorkspaceId,
    pub operation_id: OperationId,
    pub expected_revision: i64,
    pub preferences: ReadingPreferences,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RegisterAsset {
    pub contract_version: u32,
    pub workspace_id: WorkspaceId,
    pub operation_id: OperationId,
}

/// Constructed by a native adapter, never deserialized from a caller payload.
#[derive(Clone, Copy)]
pub struct RequestContext {
    trusted_ui: bool,
}
impl RequestContext {
    pub(crate) fn local_ui() -> Self {
        Self { trusted_ui: true }
    }
    pub fn unverified_external() -> Self {
        Self { trusted_ui: false }
    }
    pub(super) fn authorize(self) -> Result<()> {
        if self.trusted_ui {
            Ok(())
        } else {
            Err(AppError::new(ErrorCode::PermissionDenied))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssetInfo {
    pub digest: AssetDigest,
    pub byte_length: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct AssetDigest(String);
impl AssetDigest {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl TryFrom<String> for AssetDigest {
    type Error = AppError;
    fn try_from(value: String) -> Result<Self> {
        if value.len() == 64
            && value
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Ok(Self(value));
        }
        Err(AppError::new(ErrorCode::InvalidRequest))
    }
}
impl From<AssetDigest> for String {
    fn from(value: AssetDigest) -> Self {
        value.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum OperationOutcome {
    Preferences(PreferencesSnapshot),
    Asset(AssetInfo),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "status",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ReceiptResult {
    Applied(OperationOutcome),
    Interrupted(ErrorCode),
    Pending,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OperationReceipt {
    pub operation_id: OperationId,
    pub result: ReceiptResult,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecoveryReport {
    pub interrupted_operations: u64,
    pub staging_files: u64,
    pub orphan_assets: u64,
    pub missing_assets: u64,
    pub corrupt_assets: u64,
    pub unsafe_paths: u64,
}
impl RecoveryReport {
    pub(super) fn blocks_writes(&self) -> bool {
        [self.missing_assets, self.corrupt_assets, self.unsafe_paths]
            .into_iter()
            .any(|count| count > 0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkspaceStatus {
    pub contract_version: u32,
    pub schema_version: i64,
    pub preferences: PreferencesSnapshot,
    pub recovery: RecoveryReport,
    pub sqlite_version: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ids_reject_paths_nil_and_noncanonical_values() {
        for value in [
            "../db",
            "00000000-0000-0000-0000-000000000000",
            "22222222-2222-4222-8222-22222222ABCD",
        ] {
            assert!(OperationId::try_from(value.to_owned()).is_err(), "{value}");
        }
        let id = OperationId::generate();
        assert_eq!(
            serde_json::from_str::<OperationId>(&serde_json::to_string(&id).unwrap()).unwrap(),
            id
        );
    }
    #[test]
    fn preference_wire_rejects_unknown_fields_and_unavailable_scales() {
        assert!(
            serde_json::from_value::<ReadingPreferences>(
                serde_json::json!({"theme":"dark","textScale":200,"principal":"owner"})
            )
            .is_err()
        );
        for value in [0, 99, 101, 201, u16::MAX] {
            assert!(TextScale::try_from(value).is_err());
        }
        for value in [100, 125, 150, 200] {
            assert_eq!(TextScale::try_from(value).unwrap().get(), value);
        }
    }
}
