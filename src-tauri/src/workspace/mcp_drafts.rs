//! Durable draft-only requests for a user-approved local MCP connection.
use super::{
    ActivityRevision, AppError, ErrorCode, LessonDraft, LessonOperation, LessonPreview,
    LessonValidation, OperationId, WorkspaceId,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use uuid::{Builder, Uuid};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct DraftRequest {
    owner_key: Uuid,
    pub(super) unit_id: Uuid,
    pub(super) action: DraftAction,
}

#[derive(Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub(super) enum DraftAction {
    List,
    Materials {
        offset: u32,
    },
    Sources {
        resource_id: Uuid,
        offset: u32,
    },
    Create {
        operation_id: OperationId,
        expected_head_revision: Option<Uuid>,
    },
    Read {
        draft_id: Uuid,
    },
    Edit {
        draft_id: Uuid,
        revision: u64,
        operation_id: OperationId,
        operations: Vec<LessonOperation>,
    },
    Validate {
        draft_id: Uuid,
        revision: u64,
    },
    Preview {
        draft_id: Uuid,
        revision: u64,
    },
    Discard {
        draft_id: Uuid,
        revision: u64,
        operation_id: OperationId,
    },
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub(super) enum DraftResult {
    Draft(LessonDraft),
    List(Vec<DraftSummary>),
    Materials(serde_json::Value),
    Validation(LessonValidation),
    Preview(LessonPreview),
    Discarded { draft_id: Uuid },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DraftSummary {
    pub(super) id: String,
    pub(super) revision: u64,
}

pub(super) struct DraftActor(String);

impl DraftRequest {
    pub(super) fn actor(&self, workspace: &WorkspaceId) -> Result<DraftActor, AppError> {
        DraftActor::from_key(workspace, self.owner_key)
    }
}

impl DraftActor {
    pub(super) fn from_key(workspace: &WorkspaceId, key: Uuid) -> Result<Self, AppError> {
        if key.get_version_num() != 4 {
            return Err(AppError::new(ErrorCode::InvalidRequest));
        }
        let mut digest = Sha256::new();
        digest.update(b"memoquill:mcp:draft-owner:v1");
        digest.update(workspace.as_str().as_bytes());
        digest.update(key.as_bytes());
        Ok(DraftActor(format!("mcp:{:x}", digest.finalize())))
    }

    pub(super) fn as_str(&self) -> &str {
        &self.0
    }

    pub(super) fn operation_id(&self, operation: OperationId) -> Result<OperationId, AppError> {
        let mut digest = Sha256::new();
        digest.update(b"memoquill:mcp:draft-operation:v1");
        digest.update(self.0.as_bytes());
        digest.update(operation.as_str().as_bytes());
        let bytes = digest.finalize()[..16]
            .try_into()
            .map_err(|_| AppError::new(ErrorCode::StorageFailure))?;
        OperationId::try_from(Builder::from_random_bytes(bytes).into_uuid().to_string())
    }
}

impl DraftAction {
    pub(super) fn is_mutation(&self) -> bool {
        matches!(
            self,
            Self::Create { .. } | Self::Edit { .. } | Self::Discard { .. }
        )
    }
}

#[derive(Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub(crate) enum DraftReview {
    List,
    Inspect {
        draft_id: Uuid,
        revision: u64,
    },
    Apply {
        draft_id: Uuid,
        revision: u64,
        expected_head_revision: Option<Uuid>,
        hash: String,
        operation_id: OperationId,
    },
    Reject {
        draft_id: Uuid,
        revision: u64,
        operation_id: OperationId,
    },
}

impl DraftReview {
    pub(super) fn is_mutation(&self) -> bool {
        matches!(self, Self::Reject { .. } | Self::Apply { .. })
    }
}

#[derive(Serialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub(crate) enum DraftReviewResult {
    List(Vec<DraftSummary>),
    Inspection(Box<DraftInspection>),
    Applied(Box<super::LessonReceipt>),
    Rejected,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DraftInspection {
    pub(super) draft: LessonDraft,
    pub(super) validation: LessonValidation,
    pub(super) bindings: ResolvedDraftBindings,
    pub(super) review_hash: Option<String>,
}

#[derive(Default, Serialize)]
pub(crate) struct ResolvedDraftBindings {
    pub(super) resources: BTreeMap<String, super::SavedDocument>,
    pub(super) activities: BTreeMap<String, ActivityRevision>,
    pub(super) sources: BTreeMap<String, super::SourceBindingRevision>,
}
