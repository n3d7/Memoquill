use crate::workspace::ActivityRevision;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StudyMode {
    Practice,
    Assessment,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StudyPhase {
    #[default]
    Unspecified,
    Independent,
    Guided,
    WorkedExample,
    Completion,
    PreInstruction,
    DelayedCheck,
    ChangedContext,
}
impl StudyPhase {
    fn is_unspecified(&self) -> bool {
        *self == Self::Unspecified
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DeclaredAssistance {
    #[default]
    Unknown,
    None,
    Present,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StudyHelp {
    pub hints_revealed: u16,
    pub solution_revealed: bool,
    pub external: DeclaredAssistance,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum StudyResponse {
    Text {
        value: String,
    },
    #[serde(rename_all = "camelCase")]
    Choice {
        option_id: String,
    },
    #[serde(rename_all = "camelCase")]
    Procedure {
        completed_step_ids: Vec<String>,
        observations: String,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StudyDraft {
    pub response: Option<StudyResponse>,
    pub help: StudyHelp,
    /// First acknowledged draft save, not measured active learning time.
    #[serde(default)]
    pub first_saved_at_unix_ms: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StudyItem {
    pub binding_id: String,
    pub activity: ActivityRevision,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StudySession {
    pub contract_version: u32,
    pub id: Uuid,
    pub unit_id: String,
    pub composition_revision_id: String,
    pub mode: StudyMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assessment_id: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding_ids: Option<Vec<String>>,
    #[serde(default)]
    pub phase: StudyPhase,
    #[serde(default)]
    pub started_at_unix_ms: Option<i64>,
    pub revision: i64,
    pub cursor: usize,
    pub items: Vec<StudyItem>,
    pub draft: StudyDraft,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AssessmentOutcome {
    Correct,
    Incorrect,
    Unknown,
    Disputed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum JudgmentSource {
    Deterministic,
    SelfReport,
    /// Reserved contract: external-agent writes require M6 scoped policy.
    Model,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum AssessorIdentity {
    Local,
    Learner,
    #[serde(rename_all = "camelCase")]
    Model {
        provider: String,
        model: String,
        model_version: String,
        prompt_version: String,
        rubric_version: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct JudgmentProvenance {
    pub activity_revision_id: String,
    pub composition_revision_id: String,
    pub definition_sha256: String,
    pub assessor: AssessorIdentity,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum JudgmentUncertainty {
    #[default]
    NotEstimated,
    NotApplicable,
    #[serde(rename_all = "camelCase")]
    Reported {
        confidence_basis_points: u16,
        note: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StudyAttempt {
    pub id: Uuid,
    pub session_id: Uuid,
    pub item_index: usize,
    pub activity_revision_id: String,
    pub response: StudyResponse,
    pub help: StudyHelp,
    pub self_rating: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recall_rating: Option<u8>,
    #[serde(default)]
    pub first_saved_at_unix_ms: Option<i64>,
    #[serde(default)]
    pub submitted_at_unix_ms: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssessmentJudgment {
    pub id: Uuid,
    pub attempt_id: Uuid,
    pub source: JudgmentSource,
    pub outcome: AssessmentOutcome,
    pub assessor_version: String,
    pub supersedes: Option<Uuid>,
    /// Missing on older records; never backfilled with invented provenance.
    #[serde(default)]
    pub provenance: Option<JudgmentProvenance>,
    #[serde(default)]
    pub uncertainty: JudgmentUncertainty,
    #[serde(default)]
    pub recorded_at_unix_ms: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StudyView {
    pub session: StudySession,
    pub attempts: Vec<StudyAttempt>,
    pub judgments: Vec<AssessmentJudgment>,
    pub changed_content: bool,
    pub feedback_available: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StudyMutation {
    pub operation_id: Uuid,
    pub session_id: Uuid,
    pub expected_revision: i64,
    pub action: StudyAction,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum StudyAction {
    #[serde(rename_all = "camelCase")]
    StartScopedAssessment {
        assessment_id: Uuid,
        part_index: usize,
    },
    #[serde(rename_all = "camelCase")]
    Start {
        unit_id: String,
        composition_revision_id: String,
        mode: StudyMode,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        binding_ids: Option<Vec<String>>,
        #[serde(default, skip_serializing_if = "StudyPhase::is_unspecified")]
        phase: StudyPhase,
    },
    #[serde(rename_all = "camelCase")]
    SaveResponse {
        response: StudyResponse,
        external_assistance: DeclaredAssistance,
    },
    RevealHint,
    RevealSolution,
    #[serde(rename_all = "camelCase")]
    Submit {
        self_rating: Option<u8>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        recall_rating: Option<u8>,
        self_outcome: Option<AssessmentOutcome>,
    },
    #[serde(rename_all = "camelCase")]
    CorrectSelfJudgment {
        attempt_id: Uuid,
        supersedes: Uuid,
        outcome: AssessmentOutcome,
    },
}
