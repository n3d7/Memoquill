use super::*;
use crate::workspace::{
    AssessmentOutcome, Composition, DeclaredAssistance, JudgmentSource, RequestContext, StudyMode,
    StudyPhase, StudySession, database::Database, migrations::checksum, operations,
};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use std::collections::{BTreeMap, HashSet};
use uuid::Uuid;

type ScopedUnit = (String, String, String);
type ResolvedScope = (String, Vec<ScopedUnit>);
const POLICY: &str = "afterAllPartsComplete";
fn uuid(id: Uuid) -> Result<()> {
    if id.get_version_num() == 4 {
        Ok(())
    } else {
        Err(invalid())
    }
}
fn string_id(id: &str) -> Result<()> {
    let value = Uuid::parse_str(id).map_err(|_| invalid())?;
    uuid(value)?;
    if value.to_string() != id {
        return Err(invalid());
    }
    Ok(())
}
fn encode(value: &impl serde::Serialize) -> Result<String> {
    serde_json::to_string(value).map_err(|_| invalid())
}
fn decode<T: serde::de::DeserializeOwned>(data: String, digest: String, limit: usize) -> Result<T> {
    if data.len() > limit || checksum(data.as_bytes()) != digest {
        return Err(corrupt());
    }
    serde_json::from_str(&data).map_err(|_| corrupt())
}
fn load(connection: &Connection, id: Uuid) -> Result<FrozenAssessment> {
    let row: Option<(String, String)> = connection
        .query_row(
            "SELECT data,digest FROM assessment_runs WHERE id=?1",
            [id.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let (data, digest) = row.ok_or_else(invalid)?;
    let run: FrozenAssessment = decode(data, digest, 262144)?;
    if run.id != id {
        return Err(corrupt());
    }
    validate_structure(&run).map_err(|_| corrupt())?;
    Ok(run)
}
fn validate_structure(run: &FrozenAssessment) -> Result<()> {
    uuid(run.id)?;
    match run.scope {
        AssessmentScope::Subject { id } | AssessmentScope::Topic { id } => uuid(id)?,
    }
    if run.contract_version != 1
        || run.feedback_policy != POLICY
        || run.title.len() > 4096
        || run.parts.is_empty()
        || run.parts.len() > 64
        || run.parts.iter().map(|p| p.items.len()).sum::<usize>() > 256
        || run.omissions.len() > 256
        || run
            .created_at_unix_ms
            .is_some_and(|v| !(0..=crate::workspace::types::MAX_REVISION).contains(&v))
    {
        return Err(invalid());
    }
    let mut sessions = HashSet::new();
    let mut units = HashSet::new();
    for part in &run.parts {
        uuid(part.session_id)?;
        string_id(&part.unit_id)?;
        string_id(&part.composition_revision_id)?;
        if !sessions.insert(part.session_id)
            || !units.insert(&part.unit_id)
            || part.items.is_empty()
            || part.items.len() > 64
            || part.objective.len() > 16384
            || part.unit_title.len() > 4096
        {
            return Err(invalid());
        }
        let mut bindings = HashSet::new();
        for pin in &part.items {
            string_id(&pin.activity_revision_id)?;
            if pin.binding_id.is_empty()
                || pin.binding_id.len() > 128
                || !bindings.insert(&pin.binding_id)
                || !["recall", "singleChoice", "constructed", "procedure"]
                    .contains(&pin.family.as_str())
            {
                return Err(invalid());
            }
        }
    }
    for omission in &run.omissions {
        string_id(&omission.unit_id)?;
        if omission
            .binding_id
            .as_ref()
            .is_some_and(|id| id.is_empty() || id.len() > 128)
            || ![
                "unsupportedActivity",
                "instructionalSupport",
                "noPublishedLesson",
                "noAssessableActivity",
            ]
            .contains(&omission.reason.as_str())
        {
            return Err(invalid());
        }
    }
    Ok(())
}
fn pins(
    connection: &Connection,
    unit: &str,
    composition: &Composition,
) -> Result<(Vec<AssessmentPin>, Vec<AssessmentOmission>)> {
    let mut pins = Vec::new();
    let mut omissions = Vec::new();
    for binding in composition
        .bindings
        .iter()
        .filter(|b| b.target_kind == "activity")
    {
        let (data, digest): (String, String) = connection.query_row(
            "SELECT definition,digest FROM lesson_activity_revisions WHERE id=?1 AND unit_id=?2",
            params![binding.revision_id, unit],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let definition: serde_json::Value = decode(data, digest, 65536)?;
        let family = definition["family"].as_str().unwrap_or("");
        let supported = (definition["version"] == 1
            && ["recall", "singleChoice", "constructed", "procedure"].contains(&family))
            || definition["version"] == 2 && family == "recall";
        if !supported {
            omissions.push(AssessmentOmission {
                unit_id: unit.into(),
                binding_id: Some(binding.binding_id.clone()),
                reason: if family == "constructed" && definition["version"] == 2 {
                    "instructionalSupport"
                } else {
                    "unsupportedActivity"
                }
                .into(),
            });
        } else {
            pins.push(AssessmentPin {
                binding_id: binding.binding_id.clone(),
                activity_revision_id: binding.revision_id.clone(),
                family: family.into(),
            });
        }
    }
    Ok((pins, omissions))
}
fn scope_units(connection: &Connection, scope: &AssessmentScope) -> Result<ResolvedScope> {
    let (id, is_subject) = match scope {
        AssessmentScope::Subject { id } => (*id, true),
        AssessmentScope::Topic { id } => (*id, false),
    };
    uuid(id)?;
    let title: Option<String> = if is_subject {
        connection
            .query_row(
                "SELECT title FROM subjects WHERE id=?1 AND archived=0",
                [id.to_string()],
                |r| r.get(0),
            )
            .optional()?
    } else {
        connection.query_row("SELECT c.title FROM collections c JOIN subjects s ON s.id=c.subject_id WHERE c.id=?1 AND c.archived=0 AND s.archived=0",[id.to_string()],|r|r.get(0)).optional()?
    };
    let title = title.ok_or_else(invalid)?;
    let query = if is_subject {
        "WITH RECURSIVE scope(id) AS (SELECT id FROM collections WHERE subject_id=?1 AND parent_id IS NULL AND archived=0 UNION SELECT c.id FROM collections c JOIN scope p ON c.parent_id=p.id WHERE c.archived=0) SELECT DISTINCT u.id,u.title,u.objective FROM units u JOIN placements p ON p.unit_id=u.id JOIN scope s ON s.id=p.collection_id WHERE u.archived=0 AND p.archived=0 ORDER BY u.id LIMIT 20001"
    } else {
        "WITH RECURSIVE scope(id) AS (SELECT id FROM collections WHERE id=?1 AND archived=0 UNION SELECT c.id FROM collections c JOIN scope p ON c.parent_id=p.id WHERE c.archived=0) SELECT DISTINCT u.id,u.title,u.objective FROM units u JOIN placements p ON p.unit_id=u.id JOIN scope s ON s.id=p.collection_id WHERE u.archived=0 AND p.archived=0 ORDER BY u.id LIMIT 20001"
    };
    let rows = connection
        .prepare(query)?
        .query_map([id.to_string()], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    if rows.len() > 20000 {
        return Err(AppError::new(ErrorCode::BudgetExceeded));
    }
    Ok((title, rows))
}
pub(in crate::workspace) fn reserved(connection: &Connection, session: Uuid) -> Result<bool> {
    connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM assessment_parts WHERE session_id=?1)",
            [session.to_string()],
            |r| r.get(0),
        )
        .map_err(Into::into)
}
pub(in crate::workspace) fn resolve_start(
    connection: &Connection,
    id: Uuid,
    index: usize,
    session: Uuid,
) -> Result<AssessmentPart> {
    let run = load(connection, id)?;
    let part = run.parts.get(index).ok_or_else(invalid)?;
    if part.session_id != session {
        return Err(invalid());
    }
    Ok(part.clone())
}
fn validate_binding(
    run: &FrozenAssessment,
    part: &AssessmentPart,
    session: &StudySession,
) -> Result<()> {
    if session.id != part.session_id
        || session.assessment_id != Some(run.id)
        || session.mode != StudyMode::Assessment
        || session.phase != StudyPhase::Independent
        || session.unit_id != part.unit_id
        || session.composition_revision_id != part.composition_revision_id
        || session.binding_ids.as_ref()
            != Some(
                &part
                    .items
                    .iter()
                    .map(|p| p.binding_id.clone())
                    .collect::<Vec<_>>(),
            )
        || session.items.len() != part.items.len()
        || !session
            .items
            .iter()
            .zip(&part.items)
            .all(|(s, p)| s.binding_id == p.binding_id && s.activity.id == p.activity_revision_id)
    {
        return Err(corrupt());
    }
    Ok(())
}
pub(in crate::workspace) fn feedback_ready(connection: &Connection, id: Uuid) -> Result<bool> {
    let run = load(connection, id)?;
    for part in &run.parts {
        let row: Option<(String, String)> = connection
            .query_row(
                "SELECT data,digest FROM study_sessions WHERE id=?1",
                [part.session_id.to_string()],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        let Some((data, digest)) = row else {
            return Ok(false);
        };
        let session: StudySession = decode(data, digest, 1048576)?;
        validate_binding(&run, part, &session)?;
        if session.cursor != session.items.len() {
            return Ok(false);
        }
    }
    Ok(true)
}
impl Database {
    pub(in crate::workspace) fn assessment_create(
        &mut self,
        context: RequestContext,
        request: &CreateAssessment,
    ) -> Result<FrozenAssessment> {
        context.authorize()?;
        uuid(request.operation_id)?;
        uuid(request.assessment_id)?;
        let raw = encode(request)?;
        let hash = checksum(raw.as_bytes());
        let prior: Option<(String, String, String)> = self
            .connection
            .query_row(
                "SELECT request_hash,data,digest FROM assessment_runs WHERE operation_id=?1",
                [request.operation_id.to_string()],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?;
        if let Some((previous, data, digest)) = prior {
            if previous != hash {
                return Err(AppError::new(ErrorCode::RevisionConflict));
            }
            return decode(data, digest, 262144);
        }
        let exists: bool = self.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM assessment_runs WHERE id=?1)",
            [request.assessment_id.to_string()],
            |r| r.get(0),
        )?;
        if exists {
            return Err(AppError::new(ErrorCode::RevisionConflict));
        }
        let (title, units) = scope_units(&self.connection, &request.scope)?;
        let mut parts = Vec::new();
        let mut omitted = Vec::new();
        for (unit, unit_title, objective) in units {
            if parts.len() > 64 || omitted.len() > 256 {
                return Err(AppError::new(ErrorCode::BudgetExceeded));
            }
            let Some(saved) = self.lesson_read(context, &unit, None)? else {
                omitted.push(AssessmentOmission {
                    unit_id: unit,
                    binding_id: None,
                    reason: "noPublishedLesson".into(),
                });
                continue;
            };
            let (items, exclusions) = pins(&self.connection, &unit, &saved.composition)?;
            omitted.extend(exclusions);
            if items.is_empty() {
                omitted.push(AssessmentOmission {
                    unit_id: unit,
                    binding_id: None,
                    reason: "noAssessableActivity".into(),
                });
                continue;
            }
            parts.push(AssessmentPart {
                session_id: Uuid::new_v4(),
                unit_id: unit,
                unit_title,
                objective,
                composition_revision_id: saved.id,
                items,
            });
        }
        let created = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .and_then(|v| i64::try_from(v.as_millis()).ok())
            .filter(|v| *v <= crate::workspace::types::MAX_REVISION);
        let run = FrozenAssessment {
            contract_version: 1,
            id: request.assessment_id,
            scope: request.scope.clone(),
            title,
            feedback_policy: POLICY.into(),
            created_at_unix_ms: created,
            parts,
            omissions: omitted,
        };
        validate_structure(&run)?;
        let data = encode(&run)?;
        if data.len() > 262144 {
            return Err(AppError::new(ErrorCode::BudgetExceeded));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute(
            "INSERT INTO assessment_runs VALUES(?1,?2,?3,?4,?5,?6)",
            params![
                run.id.to_string(),
                request.operation_id.to_string(),
                raw,
                hash,
                data,
                checksum(data.as_bytes())
            ],
        )?;
        for (index, part) in run.parts.iter().enumerate() {
            tx.execute(
                "INSERT INTO assessment_parts VALUES(?1,?2,?3,?4,?5)",
                params![
                    part.session_id.to_string(),
                    run.id.to_string(),
                    index as i64,
                    part.unit_id,
                    part.composition_revision_id
                ],
            )?;
        }
        crate::workspace::faults::check(crate::workspace::faults::Phase::BeforeCommit)?;
        operations::durable_commit(tx)?;
        crate::workspace::faults::check(crate::workspace::faults::Phase::AfterCommit)
            .map_err(|_| AppError::new(ErrorCode::OutcomeUnconfirmed))?;
        Ok(run)
    }
    pub(in crate::workspace) fn assessment_list(
        &self,
        context: RequestContext,
        offset: u64,
    ) -> Result<Vec<AssessmentSummary>> {
        context.authorize()?;
        let ids = self
            .connection
            .prepare("SELECT id FROM assessment_runs ORDER BY rowid DESC LIMIT 64 OFFSET ?1")?
            .query_map([i64::try_from(offset).map_err(|_| invalid())?], |r| {
                r.get::<_, String>(0)
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        ids.into_iter()
            .map(|id| {
                let run = load(
                    &self.connection,
                    Uuid::parse_str(&id).map_err(|_| corrupt())?,
                )?;
                Ok(AssessmentSummary {
                    id: run.id,
                    title: run.title,
                    part_count: run.parts.len(),
                    item_count: run.parts.iter().map(|p| p.items.len()).sum(),
                })
            })
            .collect()
    }
    pub(in crate::workspace) fn assessment_read(
        &self,
        context: RequestContext,
        id: Uuid,
    ) -> Result<AssessmentView> {
        context.authorize()?;
        uuid(id)?;
        let run = load(&self.connection, id)?;
        let feedback = feedback_ready(&self.connection, id)?;
        let mut statuses = Vec::new();
        let mut coverage = BTreeMap::<String, OutcomeCoverage>::new();
        for part in &run.parts {
            let exists: bool = self.connection.query_row(
                "SELECT EXISTS(SELECT 1 FROM study_sessions WHERE id=?1)",
                [part.session_id.to_string()],
                |r| r.get(0),
            )?;
            let view = if exists {
                let v = self.study_read(context, part.session_id)?;
                validate_binding(&run, part, &v.session)?;
                Some(v)
            } else {
                None
            };
            let current: Option<String> = self
                .connection
                .query_row(
                    "SELECT revision_id FROM lesson_heads WHERE unit_id=?1",
                    [&part.unit_id],
                    |r| r.get(0),
                )
                .optional()?;
            statuses.push(AssessmentPartStatus {
                session_id: part.session_id,
                started: exists,
                completed: view.as_ref().map_or(0, |v| v.session.cursor),
                total: part.items.len(),
                changed_content: current.as_deref() != Some(&part.composition_revision_id),
            });
            for (index, pin) in part.items.iter().enumerate() {
                let outcome = match pin.family.as_str() {
                    "recall" => "recall",
                    "singleChoice" => "recognition",
                    "constructed" => "constructedResponse",
                    _ => "practicalSelfReport",
                }
                .to_owned();
                let row = coverage
                    .entry(outcome.clone())
                    .or_insert_with(|| OutcomeCoverage {
                        outcome,
                        total: 0,
                        attempted: 0,
                        unattempted: 0,
                        declared_unaided: 0,
                        external_unknown: 0,
                        external_present: 0,
                        feedback: feedback.then(OutcomeCounts::default),
                    });
                row.total += 1;
                let Some(view) = &view else {
                    row.unattempted += 1;
                    continue;
                };
                let Some(attempt) = view.attempts.get(index) else {
                    row.unattempted += 1;
                    continue;
                };
                row.attempted += 1;
                match attempt.help.external {
                    DeclaredAssistance::None
                        if attempt.help.hints_revealed == 0 && !attempt.help.solution_revealed =>
                    {
                        row.declared_unaided += 1
                    }
                    DeclaredAssistance::Unknown => row.external_unknown += 1,
                    DeclaredAssistance::Present => row.external_present += 1,
                    _ => {}
                }
                if let Some(counts) = &mut row.feedback {
                    let judgments: Vec<_> = view
                        .judgments
                        .iter()
                        .filter(|j| j.attempt_id == attempt.id)
                        .collect();
                    match judgments
                        .iter()
                        .find(|j| j.source == JudgmentSource::Deterministic)
                        .map(|j| j.outcome)
                    {
                        Some(AssessmentOutcome::Correct) => counts.deterministic_correct += 1,
                        Some(AssessmentOutcome::Incorrect) => counts.deterministic_incorrect += 1,
                        _ => counts.deterministic_unknown += 1,
                    }
                    let last = judgments.iter().find(|j| {
                        j.source == JudgmentSource::SelfReport
                            && !judgments.iter().any(|next| next.supersedes == Some(j.id))
                    });
                    match last.map(|j| j.outcome) {
                        Some(AssessmentOutcome::Correct) => counts.self_correct += 1,
                        Some(AssessmentOutcome::Incorrect) => counts.self_incorrect += 1,
                        Some(AssessmentOutcome::Disputed) => counts.self_disputed += 1,
                        _ => counts.self_unknown += 1,
                    }
                }
            }
        }
        Ok(AssessmentView {
            assessment: run,
            parts: statuses,
            feedback_available: feedback,
            coverage: coverage.into_values().collect(),
        })
    }
}
pub(in crate::workspace) fn validate_native_data(connection: &Connection) -> Result<()> {
    let rows = connection
        .prepare("SELECT id,operation_id,request,request_hash FROM assessment_runs ORDER BY rowid")?
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let mut expected = BTreeMap::new();
    for (id, operation, request, hash) in rows {
        if request.len() > 4096 || checksum(request.as_bytes()) != hash {
            return Err(corrupt());
        }
        let request: CreateAssessment = serde_json::from_str(&request).map_err(|_| corrupt())?;
        uuid(request.operation_id).map_err(|_| corrupt())?;
        let run = load(connection, Uuid::parse_str(&id).map_err(|_| corrupt())?)?;
        if operation != request.operation_id.to_string()
            || run.id != request.assessment_id
            || run.scope != request.scope
        {
            return Err(corrupt());
        }
        let scope_exists: bool = match run.scope {
            AssessmentScope::Subject { id } => connection.query_row(
                "SELECT EXISTS(SELECT 1 FROM subjects WHERE id=?1)",
                [id.to_string()],
                |r| r.get(0),
            )?,
            AssessmentScope::Topic { id } => connection.query_row(
                "SELECT EXISTS(SELECT 1 FROM collections WHERE id=?1)",
                [id.to_string()],
                |r| r.get(0),
            )?,
        };
        if !scope_exists {
            return Err(corrupt());
        }
        for (index, part) in run.parts.iter().enumerate() {
            let (data, digest): (String, String) = connection.query_row(
                "SELECT data,digest FROM lesson_revisions WHERE id=?1 AND unit_id=?2",
                params![part.composition_revision_id, part.unit_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?;
            let composition: Composition = decode(data, digest, 1048576)?;
            if pins(connection, &part.unit_id, &composition)?.0 != part.items {
                return Err(corrupt());
            }
            if expected
                .insert(
                    part.session_id.to_string(),
                    (
                        run.id.to_string(),
                        index as i64,
                        part.unit_id.clone(),
                        part.composition_revision_id.clone(),
                    ),
                )
                .is_some()
            {
                return Err(corrupt());
            }
            let row: Option<(String, String)> = connection
                .query_row(
                    "SELECT data,digest FROM study_sessions WHERE id=?1",
                    [part.session_id.to_string()],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()?;
            if let Some((data, digest)) = row {
                let session: StudySession = decode(data, digest, 1048576)?;
                validate_binding(&run, part, &session)?;
            }
        }
    }
    let parts=connection.prepare("SELECT session_id,assessment_id,part_index,unit_id,composition_id FROM assessment_parts")?.query_map([],|r|Ok((r.get::<_,String>(0)?,(r.get::<_,String>(1)?,r.get::<_,i64>(2)?,r.get::<_,String>(3)?,r.get::<_,String>(4)?))))?.collect::<rusqlite::Result<BTreeMap<_,_>>>()?;
    if expected != parts {
        return Err(corrupt());
    }
    let sessions = connection
        .prepare("SELECT id,data,digest FROM study_sessions")?
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    for (id, data, digest) in sessions {
        let session: StudySession = decode(data, digest, 1048576)?;
        if session.assessment_id.map(|id| id.to_string()) != parts.get(&id).map(|p| p.0.clone()) {
            return Err(corrupt());
        }
    }
    Ok(())
}
