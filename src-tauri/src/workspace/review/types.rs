use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecallMemory {
    pub stability: f32,
    pub difficulty: f32,
    pub stability_fast: f32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PracticalReminder {
    pub id: Uuid,
    pub unit_id: String,
    pub activity_revision_id: String,
    pub due_day: i32,
    pub interval_days: Option<u16>,
    pub archived: bool,
    pub completed_at_unix_ms: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReviewPlan {
    pub contract_version: u32,
    pub revision: i64,
    pub generation: i64,
    pub time_zone: String,
    pub day_start_hour: u8,
    pub daily_new_limit: u16,
    pub daily_review_limit: u16,
    pub retention_basis_points: u16,
    pub reminders: Vec<PracticalReminder>,
}
impl Default for ReviewPlan {
    fn default() -> Self {
        Self {
            contract_version: 1,
            revision: 0,
            generation: 1,
            time_zone: "Etc/UTC".into(),
            day_start_hour: 4,
            daily_new_limit: 10,
            daily_review_limit: 50,
            retention_basis_points: 9000,
            reminders: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReviewMutation {
    pub operation_id: Uuid,
    pub expected_revision: i64,
    pub action: ReviewAction,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum ReviewAction {
    #[serde(rename_all = "camelCase")]
    SetPolicy {
        time_zone: String,
        day_start_hour: u8,
        daily_new_limit: u16,
        daily_review_limit: u16,
        retention_basis_points: u16,
    },
    #[serde(rename_all = "camelCase")]
    AddReminder {
        id: Uuid,
        unit_id: String,
        activity_revision_id: String,
        due_day: i32,
        interval_days: Option<u16>,
    },
    #[serde(rename_all = "camelCase")]
    CompleteReminder { id: Uuid },
    #[serde(rename_all = "camelCase")]
    ArchiveReminder { id: Uuid },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecallReviewEvent {
    pub contract_version: u32,
    pub id: Uuid,
    pub attempt_id: Uuid,
    pub session_id: Uuid,
    pub unit_id: String,
    pub activity_revision_id: String,
    pub prior_event_id: Option<Uuid>,
    pub generation: i64,
    pub plan_revision: i64,
    pub algorithm: String,
    pub parameters: Vec<f32>,
    pub parameter_sha256: String,
    pub retention_basis_points: u16,
    pub time_zone: String,
    pub time_zone_rules: String,
    pub day_start_hour: u8,
    pub reviewed_at_unix_ms: i64,
    pub study_day: i32,
    pub rating: u8,
    pub memory: RecallMemory,
    pub raw_interval_days: f32,
    pub interval_days: u32,
    pub due_day: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecallQueueItem {
    pub unit_id: String,
    pub composition_revision_id: String,
    pub binding_id: String,
    pub activity_revision_id: String,
    pub prompt: String,
    pub due_day: Option<i32>,
    pub due_date: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReviewOverview {
    pub plan: ReviewPlan,
    pub now_unix_ms: i64,
    pub today: i32,
    pub today_date: String,
    pub time_zone_rules: String,
    pub clock_rollback: bool,
    pub new_used: u32,
    pub reviews_used: u32,
    pub due_total: u32,
    pub new_total: u32,
    pub offered: Vec<RecallQueueItem>,
}
