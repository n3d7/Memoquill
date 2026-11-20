//! Read-only projections of canonical evidence. No persisted score or mastery model.
use super::*;
use crate::workspace::{Binding, RequestContext, database::Database};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

const MAX_SESSIONS: usize = 1024;
const MAX_RECORDS: usize = 65_536;
const PAGE_SIZE: usize = 32;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressJudgmentCount {
    pub source: JudgmentSource,
    pub outcome: AssessmentOutcome,
    pub count: usize,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressCoverage {
    pub outcome: String,
    pub current_activity_bindings: usize,
    pub evidenced_current_activity_bindings: usize,
    pub attempts: usize,
    pub previous_activity_attempts: usize,
    pub feedback_withheld_attempts: usize,
    pub unassessed_attempts: usize,
    pub judgments: Vec<ProgressJudgmentCount>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEvidence {
    pub attempt: StudyAttempt,
    pub binding_id: String,
    pub composition_revision_id: String,
    pub outcome: String,
    pub mode: StudyMode,
    pub phase: StudyPhase,
    pub current_activity: bool,
    pub changed_composition: bool,
    pub feedback_available: bool,
    pub judgments: Vec<AssessmentJudgment>,
    pub sources: Vec<Binding>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonProgress {
    pub contract_version: u32,
    pub unit_id: String,
    pub current_composition_revision_id: Option<String>,
    pub session_count: usize,
    pub total_attempts: usize,
    pub offset: u64,
    pub next_offset: Option<u64>,
    pub coverage: Vec<ProgressCoverage>,
    pub evidence: Vec<ProgressEvidence>,
}
fn outcome(family: Option<&str>) -> &'static str {
    match family {
        Some("recall") => "recall",
        Some("singleChoice") => "recognition",
        Some("constructed") => "constructedResponse",
        Some("procedure") => "practicalSelfReport",
        _ => "unknown",
    }
}
fn budget() -> AppError {
    AppError::new(ErrorCode::BudgetExceeded)
}
impl Database {
    pub(in crate::workspace) fn study_progress(
        &self,
        context: RequestContext,
        unit_id: &str,
        offset: u64,
    ) -> Result<LessonProgress> {
        context.authorize()?;
        self.lesson_list_drafts(context, unit_id)?;
        let offset_index = usize::try_from(offset).map_err(|_| invalid())?;
        let current = self.lesson_read(context, unit_id, None)?;
        let mut bindings = BTreeMap::new();
        let mut coverage = BTreeMap::<String, ProgressCoverage>::new();
        if let Some(saved) = &current {
            for binding in &saved.composition.bindings {
                if binding.target_kind != "activity" {
                    continue;
                }
                let activity = self.lesson_activity(context, unit_id, &binding.revision_id)?;
                let key = outcome(
                    activity
                        .as_ref()
                        .and_then(|a| a.definition["family"].as_str()),
                );
                bindings.insert(binding.binding_id.clone(), binding.revision_id.clone());
                let row = coverage.entry(key.into()).or_default();
                row.outcome = key.into();
                row.current_activity_bindings += 1;
            }
        }
        // Do not silently turn a truncated scan into complete coverage. Explicit
        // budgets bound canonical JSON validation and return a non-passing result.
        let ids = self
            .connection
            .prepare(
                "SELECT id FROM study_sessions WHERE unit_id=?1 ORDER BY rowid DESC LIMIT 1025",
            )?
            .query_map([unit_id], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        if ids.len() > MAX_SESSIONS {
            return Err(budget());
        }
        // Count raw canonical work before study_read filters embargoed judgments.
        // Otherwise hidden feedback could bypass the cumulative projection budget.
        let (raw_records, raw_bytes): (i64, i64) = self.connection.query_row(
            "SELECT COALESCE(sum(records),0), COALESCE(sum(length(CAST(data AS BLOB))),0) FROM (
                SELECT 0 AS records, data FROM study_sessions WHERE unit_id=?1
                UNION ALL SELECT 1, a.data FROM study_attempts a JOIN study_sessions s ON s.id=a.session_id WHERE s.unit_id=?1
                UNION ALL SELECT 1, j.data FROM study_judgments j JOIN study_attempts a ON a.id=j.attempt_id JOIN study_sessions s ON s.id=a.session_id WHERE s.unit_id=?1
            )", [unit_id], |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        if raw_records > MAX_RECORDS as i64 || raw_bytes > 16 * 1024 * 1024 {
            return Err(budget());
        }
        let mut evidence = Vec::new();
        let mut total_attempts = 0;
        let mut records = 0;
        let mut bytes = 0;
        let mut observed = BTreeSet::new();
        for id in &ids {
            let view = self.study_read(context, Uuid::parse_str(id).map_err(|_| corrupt())?)?;
            if view.session.unit_id != unit_id {
                return Err(corrupt());
            }
            bytes += serde_json::to_vec(&view).map_err(|_| corrupt())?.len();
            records += view.attempts.len() + view.judgments.len();
            if records > MAX_RECORDS || bytes > 16 * 1024 * 1024 {
                return Err(budget());
            }
            let pinned = self.lesson_read(
                context,
                unit_id,
                Some(&view.session.composition_revision_id),
            )?;
            let sources: Vec<Binding> = pinned
                .as_ref()
                .map(|p| {
                    p.composition
                        .bindings
                        .iter()
                        .filter(|b| {
                            b.target_kind == "source"
                                && p.composition.source_binding_refs.contains(&b.binding_id)
                        })
                        .cloned()
                        .collect()
                })
                .unwrap_or_default();
            let superseded: BTreeSet<_> =
                view.judgments.iter().filter_map(|j| j.supersedes).collect();
            // Newest sessions first, then newest submitted item; never order by
            // wall-clock timestamps that may be missing or move backwards.
            for attempt in view.attempts.iter().rev() {
                let item = view
                    .session
                    .items
                    .get(attempt.item_index)
                    .ok_or_else(corrupt)?;
                let key = outcome(item.activity.definition["family"].as_str());
                let current_activity =
                    bindings.get(&item.binding_id) == Some(&attempt.activity_revision_id);
                let row = coverage.entry(key.into()).or_default();
                row.outcome = key.into();
                row.attempts += 1;
                if !current_activity {
                    row.previous_activity_attempts += 1;
                }
                if current_activity && observed.insert(item.binding_id.clone()) {
                    row.evidenced_current_activity_bindings += 1;
                }
                // study_read already applies the global frozen-assessment embargo.
                let judgments: Vec<_> = view
                    .judgments
                    .iter()
                    .filter(|j| j.attempt_id == attempt.id)
                    .cloned()
                    .collect();
                if !view.feedback_available {
                    row.feedback_withheld_attempts += 1;
                } else {
                    let active: Vec<_> = judgments
                        .iter()
                        .filter(|j| !superseded.contains(&j.id))
                        .collect();
                    if active.is_empty() {
                        row.unassessed_attempts += 1;
                    }
                    for judgment in active {
                        if let Some(count) = row
                            .judgments
                            .iter_mut()
                            .find(|c| c.source == judgment.source && c.outcome == judgment.outcome)
                        {
                            count.count += 1;
                        } else {
                            row.judgments.push(ProgressJudgmentCount {
                                source: judgment.source,
                                outcome: judgment.outcome,
                                count: 1,
                            });
                        }
                    }
                }
                if total_attempts >= offset_index && evidence.len() < PAGE_SIZE {
                    evidence.push(ProgressEvidence {
                        attempt: attempt.clone(),
                        binding_id: item.binding_id.clone(),
                        composition_revision_id: view.session.composition_revision_id.clone(),
                        outcome: key.into(),
                        mode: view.session.mode,
                        phase: view.session.phase,
                        current_activity,
                        changed_composition: view.changed_content,
                        feedback_available: view.feedback_available,
                        judgments,
                        sources: sources.clone(),
                    });
                }
                total_attempts += 1;
            }
        }
        if offset_index > total_attempts {
            return Err(invalid());
        }
        let result = LessonProgress {
            contract_version: 1,
            unit_id: unit_id.into(),
            current_composition_revision_id: current.map(|p| p.id),
            session_count: ids.len(),
            total_attempts,
            offset,
            next_offset: (offset_index + evidence.len() < total_attempts)
                .then_some(offset + evidence.len() as u64),
            coverage: coverage.into_values().collect(),
            evidence,
        };
        if serde_json::to_vec(&result).map_err(|_| corrupt())?.len() > 1_048_576 {
            return Err(budget());
        }
        Ok(result)
    }
}
