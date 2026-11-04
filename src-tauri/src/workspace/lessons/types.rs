use super::*;
use crate::workspace::{OperationId, WorkspaceId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Composition {
    pub contract_version: u32,
    pub unit_id: String,
    pub root_node_id: String,
    pub nodes: Vec<LessonNode>,
    pub bindings: Vec<Binding>,
    pub source_binding_refs: Vec<String>,
    pub appearance: Appearance,
    pub authoring_provenance: Provenance,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LessonNode {
    pub node_id: String,
    pub type_id: String,
    pub type_version: u32,
    pub props: Value,
    pub slots: BTreeMap<String, Vec<String>>,
    pub on: BTreeMap<String, ActionBinding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible: Option<Value>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Binding {
    pub binding_id: String,
    pub target_kind: String,
    pub target_id: String,
    pub revision_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locator: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ActionBinding {
    pub action_id: String,
    pub version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_node_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding_id: Option<String>,
    pub inputs: BTreeMap<String, Value>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Appearance {
    pub spacing: String,
    pub theme: String,
    pub typography: String,
    pub width: String,
    pub emphasis: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LessonDraft {
    pub id: String,
    pub unit_id: String,
    pub revision: u64,
    pub composition: Composition,
    pub base_head_revision: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LessonPreview {
    pub draft_id: String,
    pub draft_revision: u64,
    pub composition: Composition,
    pub hash: String,
    pub warnings: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SavedComposition {
    pub id: String,
    pub unit_id: String,
    pub composition: Composition,
    pub parent_revision_id: Option<String>,
    pub registry_contract_version: u32,
    pub hash: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LessonReceipt {
    pub operation_id: OperationId,
    pub saved: SavedComposition,
    pub actor: String,
    pub proposal_hash: String,
    pub old_head: Option<String>,
    pub new_head: String,
    pub created_revision_ids: Vec<String>,
    pub id_map: BTreeMap<String, String>,
    pub warnings: Vec<String>,
    pub committed: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ActivityRevision {
    pub id: String,
    pub activity_id: String,
    pub unit_id: String,
    pub definition: Value,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum LessonOperation {
    ReplaceUnsupportedRoot {
        node: LessonNode,
    },
    #[serde(rename_all = "camelCase")]
    CreateSourceBindingRevision {
        source_id: String,
        binding_id: String,
        locator: Option<String>,
        declared_used_ranges: Vec<String>,
    },

    InsertNode {
        parent: String,
        slot: String,
        index: usize,
        node: LessonNode,
    },
    RemoveNode {
        id: String,
    },
    MoveNode {
        id: String,
        parent: String,
        slot: String,
        index: usize,
    },
    ConfigureNode {
        id: String,
        props: Value,
    },
    ConfigureInteraction {
        id: String,
        on: BTreeMap<String, ActionBinding>,
        visible: Option<Value>,
    },
    BindReference {
        binding: Binding,
    },
    SetAppearance {
        tokens: Appearance,
    },
    #[serde(rename_all = "camelCase")]
    CreateActivityRevision {
        activity_id: String,
        expected_revision_id: Option<String>,
        binding_id: String,
        definition: Value,
    },
    #[serde(rename_all = "camelCase")]
    CreateResourceRevision {
        resource_id: String,
        expected_revision_id: Option<String>,
        title: String,
        binding_id: String,
        document: crate::document::Document,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateLessonDraft {
    pub contract_version: u32,
    pub workspace_id: WorkspaceId,
    pub unit_id: String,
    pub operation_id: OperationId,
    pub expected_head_revision: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EditLessonDraft {
    pub contract_version: u32,
    pub workspace_id: WorkspaceId,
    pub unit_id: String,
    pub draft_id: String,
    pub expected_draft_revision: u64,
    pub operation_id: OperationId,
    pub operations: Vec<LessonOperation>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LessonDraftScope {
    pub contract_version: u32,
    pub workspace_id: WorkspaceId,
    pub unit_id: String,
    pub draft_id: String,
    pub expected_draft_revision: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ApplyLessonDraft {
    pub contract_version: u32,
    pub workspace_id: WorkspaceId,
    pub unit_id: String,
    pub draft_id: String,
    pub expected_draft_revision: u64,
    pub expected_head_revision: Option<String>,
    pub preview_hash: String,
    pub operation_id: OperationId,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DiscardLessonDraft {
    pub contract_version: u32,
    pub workspace_id: WorkspaceId,
    pub unit_id: String,
    pub draft_id: String,
    pub expected_draft_revision: u64,
    pub operation_id: OperationId,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RegistryDescriptor {
    pub type_id: String,
    pub type_version: u32,
    pub slots: Vec<String>,
    pub events: Vec<String>,
    pub props: BTreeMap<String, Value>,
    pub props_schema: Value,
    pub defaults: BTreeMap<String, Value>,
    pub slot_schemas: BTreeMap<String, SlotDescriptor>,
    pub event_schemas: BTreeMap<String, EventDescriptor>,
    pub actions: Vec<ActionDescriptor>,
    pub required_capabilities: Vec<String>,
    pub valid_modes: Vec<String>,
    pub state: StateDescriptor,
    pub accessibility_contract: Vec<String>,
    pub fallback: FallbackDescriptor,
    pub supported_migrations: Vec<u32>,
    pub schemas: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceBindingRevision {
    pub id: String,
    pub unit_id: String,
    pub source_id: String,
    pub asset_digest: String,
    pub filename: String,
    pub attribution: Option<String>,
    pub license: Option<String>,
    pub origin_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locator: Option<String>,
    pub declared_used_ranges: Vec<String>,
    pub extraction_coverage: String,
    pub read_coverage: String,
    pub warnings: Vec<String>,
}

/// Safe locations identify only records already supplied by or visible to this caller.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LessonDiagnosticCode {
    InvalidProps,
    InvalidCondition,
    InvalidAction,
    InvalidReference,
    InvalidTree,
    InvalidOperation,
    UnsupportedType,
    BudgetExceeded,
    InvalidEnvelope,
    RevisionConflict,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LessonRepair {
    UseDeclaredValue,
    UseDeclaredField,
    SelectPinnedRevision,
    RemoveOrReplaceUnsupported,
    CorrectTree,
    SplitRequest,
    RefreshBase,
    UseClosedEnvelope,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LessonDiagnostic {
    pub code: LessonDiagnosticCode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_index: Option<u32>,
    pub field: String,
    pub repair: LessonRepair,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LessonValidation {
    pub draft_id: String,
    pub draft_revision: u64,
    pub valid: bool,
    pub diagnostics: Vec<LessonDiagnostic>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SlotDescriptor {
    pub minimum: u32,
    pub maximum: u32,
    pub ordered: bool,
    pub allowed_children: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EventDescriptor {
    pub payload_schema: Value,
    pub allowed_actions: Vec<String>,
    pub event_fields: BTreeMap<String, Value>,
    pub maximum_actions: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ActionDescriptor {
    pub action_id: String,
    pub version: u32,
    pub input_schema: Value,
    pub result_schema: Value,
    pub target_kinds: Vec<String>,
    pub required_capabilities: Vec<String>,
    pub valid_modes: Vec<String>,
    pub availability: BTreeMap<String, String>,
    pub side_effect_class: String,
    pub idempotency: String,
    pub event_field_sources: Vec<String>,
    pub error_codes: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StateDescriptor {
    pub scope: String,
    pub schema: Value,
    pub maximum_bytes: u32,
    pub persistent: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FallbackDescriptor {
    pub mode: String,
    pub preserves_opaque_payload: bool,
    pub activation_allowed: bool,
    pub diagnostic_code: LessonDiagnosticCode,
    pub repairs: Vec<LessonRepair>,
}
