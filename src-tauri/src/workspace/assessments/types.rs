use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum AssessmentScope {
    Subject { id: Uuid },
    Topic { id: Uuid },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateAssessment {
    pub operation_id: Uuid,
    pub assessment_id: Uuid,
    pub scope: AssessmentScope,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssessmentPin {
    pub binding_id: String,
    pub activity_revision_id: String,
    pub family: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssessmentPart {
    pub session_id: Uuid,
    pub unit_id: String,
    pub unit_title: String,
    pub objective: String,
    pub composition_revision_id: String,
    pub items: Vec<AssessmentPin>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssessmentOmission {
    pub unit_id: String,
    pub binding_id: Option<String>,
    pub reason: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FrozenAssessment {
    pub contract_version: u32,
    pub id: Uuid,
    pub scope: AssessmentScope,
    pub title: String,
    pub feedback_policy: String,
    pub created_at_unix_ms: Option<i64>,
    pub parts: Vec<AssessmentPart>,
    pub omissions: Vec<AssessmentOmission>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssessmentSummary {
    pub id: Uuid,
    pub title: String,
    pub part_count: usize,
    pub item_count: usize,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OutcomeCounts {
    pub deterministic_correct: u32,
    pub deterministic_incorrect: u32,
    pub deterministic_unknown: u32,
    pub self_correct: u32,
    pub self_incorrect: u32,
    pub self_unknown: u32,
    pub self_disputed: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OutcomeCoverage {
    pub outcome: String,
    pub total: u32,
    pub attempted: u32,
    pub unattempted: u32,
    pub declared_unaided: u32,
    pub external_unknown: u32,
    pub external_present: u32,
    pub feedback: Option<OutcomeCounts>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssessmentPartStatus {
    pub session_id: Uuid,
    pub started: bool,
    pub completed: usize,
    pub total: usize,
    pub changed_content: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssessmentView {
    pub assessment: FrozenAssessment,
    pub parts: Vec<AssessmentPartStatus>,
    pub feedback_available: bool,
    pub coverage: Vec<OutcomeCoverage>,
}
