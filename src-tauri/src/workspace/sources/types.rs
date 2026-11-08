use super::*;
use serde::{Deserialize, Serialize};

pub const MAX_SOURCE_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NewSource {
    pub id: Uuid,
    pub resource_id: ResourceId,
    pub digest: AssetDigest,
    pub filename: String,
    pub media_type: String,
    pub attribution: Option<String>,
    pub license: Option<String>,
    pub origin_url: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OriginalSource {
    pub id: Uuid,
    pub resource_id: ResourceId,
    pub digest: AssetDigest,
    pub filename: String,
    pub media_type: String,
    pub attribution: Option<String>,
    pub license: Option<String>,
    pub origin_url: Option<String>,
    pub revision: u64,
    pub byte_length: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceList {
    pub revision: u64,
    pub sources: Vec<OriginalSource>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveSource {
    pub contract_version: u32,
    pub workspace_id: WorkspaceId,
    pub operation_id: OperationId,
    pub expected_revision: u64,
    pub source: NewSource,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourcePreview {
    pub source: OriginalSource,
    pub text: Option<String>,
    pub hex: Option<String>,
    pub truncated: bool,
}
