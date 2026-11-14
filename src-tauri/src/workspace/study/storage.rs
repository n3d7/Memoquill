use super::*;
use crate::workspace::{RequestContext, database::Database, migrations::checksum, operations};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Serialize, de::DeserializeOwned};
use uuid::Uuid;

const MAX_SESSION: usize = 1_048_576;
const MAX_ITEMS: usize = 64;

fn wall_time_ms() -> Option<i64> {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_millis();
    i64::try_from(millis)
        .ok()
        .filter(|v| *v <= crate::workspace::types::MAX_REVISION)
}
fn valid_time(value: Option<i64>) -> bool {
    value.is_none_or(|v| (0..=crate::workspace::types::MAX_REVISION).contains(&v))
}
fn provenance(
    session: &StudySession,
    index: usize,
    assessor: AssessorIdentity,
) -> Result<JudgmentProvenance> {
    let activity = &session.items.get(index).ok_or_else(corrupt)?.activity;
    Ok(JudgmentProvenance {
        activity_revision_id: activity.id.clone(),
        composition_revision_id: session.composition_revision_id.clone(),
        definition_sha256: checksum(encode(&activity.definition)?.as_bytes()),
        assessor,
    })
}

fn encode(value: &impl Serialize) -> Result<String> {
    serde_json::to_string(value).map_err(|_| invalid())
}
fn decode<T: DeserializeOwned>(data: String, digest: String, limit: usize) -> Result<T> {
    if data.len() > limit || checksum(data.as_bytes()) != digest {
        return Err(corrupt());
    }
    serde_json::from_str(&data).map_err(|_| corrupt())
}
fn uuid(id: Uuid) -> Result<()> {
    if id.get_version_num() == 4 {
        Ok(())
    } else {
        Err(invalid())
    }
}
fn load(connection: &Connection, id: Uuid) -> Result<StudySession> {
    let row: Option<(String, String)> = connection
        .query_row(
            "SELECT data,digest FROM study_sessions WHERE id=?1",
            [id.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let (data, digest) = row.ok_or_else(invalid)?;
    let session: StudySession = decode(data, digest, MAX_SESSION)?;
    if session.id != id {
        return Err(corrupt());
    }
    validate_session(&session)?;
    Ok(session)
}
fn instructional(item: &StudyItem) -> bool {
    item.activity.definition["family"] == "constructed" && item.activity.definition["version"] == 2
}
fn initial_draft(item: Option<&StudyItem>) -> StudyDraft {
    let mut draft = StudyDraft::default();
    draft.help.solution_revealed = item.is_some_and(instructional);
    draft
}
fn validate_help(session: &StudySession, index: usize, help: &StudyHelp) -> Result<()> {
    let item = session.items.get(index).ok_or_else(corrupt)?;
    let hints = item.activity.definition["hints"]
        .as_array()
        .ok_or_else(corrupt)?;
    if help.hints_revealed as usize > hints.len()
        || instructional(item) && (!help.solution_revealed || session.mode == StudyMode::Assessment)
        || session.mode == StudyMode::Assessment
            && (help.hints_revealed > 0 || help.solution_revealed)
    {
        return Err(corrupt());
    }
    Ok(())
}
fn validate_session(session: &StudySession) -> Result<()> {
    uuid(session.id).map_err(|_| corrupt())?;
    if let Some(id) = session.assessment_id {
        uuid(id).map_err(|_| corrupt())?;
        if session.mode != StudyMode::Assessment
            || session.phase != StudyPhase::Independent
            || session.binding_ids.is_none()
        {
            return Err(corrupt());
        }
    }
    if session.contract_version != 1
        || !valid_time(session.started_at_unix_ms)
        || !valid_time(session.draft.first_saved_at_unix_ms)
        || session.revision < 0
        || session.revision > crate::workspace::types::MAX_REVISION
        || session.binding_ids.as_ref().is_some_and(|ids| {
            ids.len() != session.items.len()
                || ids.iter().collect::<std::collections::HashSet<_>>().len() != ids.len()
                || !session
                    .items
                    .iter()
                    .all(|item| ids.contains(&item.binding_id))
        })
        || session.items.is_empty()
        || session.items.len() > MAX_ITEMS
        || session.mode == StudyMode::Assessment && session.items.iter().any(instructional)
        || session.cursor > session.items.len()
    {
        return Err(corrupt());
    }
    let mut seen = std::collections::HashSet::new();
    for item in &session.items {
        if item.activity.unit_id != session.unit_id || !seen.insert(&item.binding_id) {
            return Err(corrupt());
        }
    }
    if session.cursor < session.items.len() {
        validate_help(session, session.cursor, &session.draft.help)?;
        if let Some(response) = &session.draft.response {
            validate_response(&session.items[session.cursor], response).map_err(|_| corrupt())?;
        }
    } else if session.draft != StudyDraft::default() {
        return Err(corrupt());
    }
    Ok(())
}

impl Database {
    pub(in crate::workspace) fn study_read(
        &self,
        context: RequestContext,
        session_id: Uuid,
    ) -> Result<StudyView> {
        context.authorize()?;
        uuid(session_id)?;
        let session = load(&self.connection, session_id)?;
        let current = self.lesson_read(context, &session.unit_id, None)?;
        let changed_content = current
            .as_ref()
            .is_none_or(|saved| saved.id != session.composition_revision_id);
        let attempts = read_attempts(&self.connection, &session)?;
        let judgments = read_judgments(&self.connection, &session, &attempts)?;
        let feedback_available = if session.mode == StudyMode::Practice {
            true
        } else if let Some(id) = session.assessment_id {
            crate::workspace::assessments::feedback_ready(&self.connection, id)?
        } else {
            session.cursor == session.items.len()
        };
        let judgments = if feedback_available {
            judgments
        } else {
            Vec::new()
        };
        Ok(StudyView {
            session,
            attempts,
            judgments,
            changed_content,
            feedback_available,
        })
    }

    pub(in crate::workspace) fn study_sessions(
        &self,
        context: RequestContext,
        unit_id: &str,
        offset: u64,
    ) -> Result<Vec<StudySession>> {
        context.authorize()?;
        // Reuse existing unit ownership/identifier admission, even when there is no composition.
        self.lesson_list_drafts(context, unit_id)?;
        let ids = self
            .connection
            .prepare("SELECT id FROM study_sessions WHERE unit_id=?1 ORDER BY rowid DESC LIMIT 64 OFFSET ?2")?
            .query_map(params![unit_id, i64::try_from(offset).map_err(|_| invalid())?], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        ids.into_iter()
            .map(|id| {
                load(
                    &self.connection,
                    Uuid::parse_str(&id).map_err(|_| corrupt())?,
                )
            })
            .collect()
    }

    pub(in crate::workspace) fn study_apply(
        &mut self,
        context: RequestContext,
        request: &StudyMutation,
    ) -> Result<StudySession> {
        context.authorize()?;
        uuid(request.operation_id)?;
        uuid(request.session_id)?;
        let request_data = encode(request)?;
        if request_data.len() > 65536 {
            return Err(AppError::new(ErrorCode::BudgetExceeded));
        }
        let hash = checksum(request_data.as_bytes());
        let previous: Option<(String, String, String, String)> = self
            .connection
            .query_row(
                "SELECT session_id,request_hash,result,digest FROM study_operations WHERE id=?1",
                [request.operation_id.to_string()],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .optional()?;
        if let Some((session_id, previous_hash, data, digest)) = previous {
            if session_id != request.session_id.to_string() || previous_hash != hash {
                return Err(conflict());
            }
            return decode(data, digest, MAX_SESSION);
        }
        let scoped = if let StudyAction::StartScopedAssessment {
            assessment_id,
            part_index,
        } = &request.action
        {
            Some((
                *assessment_id,
                crate::workspace::assessments::resolve_start(
                    &self.connection,
                    *assessment_id,
                    *part_index,
                    request.session_id,
                )?,
            ))
        } else {
            None
        };
        if matches!(&request.action, StudyAction::Start { .. })
            && crate::workspace::assessments::reserved(&self.connection, request.session_id)?
        {
            return Err(invalid());
        }
        let start = scoped.as_ref().map(|(_, part)| StudyAction::Start {
            unit_id: part.unit_id.clone(),
            composition_revision_id: part.composition_revision_id.clone(),
            mode: StudyMode::Assessment,
            phase: StudyPhase::Independent,
            binding_ids: Some(part.items.iter().map(|p| p.binding_id.clone()).collect()),
        });
        let effective = start.as_ref().unwrap_or(&request.action);
        let mut session = if let StudyAction::Start {
            unit_id,
            composition_revision_id,
            mode,
            phase,
            binding_ids,
        } = effective
        {
            if request.expected_revision != 0 {
                return Err(conflict());
            }
            let exists: bool = self.connection.query_row(
                "SELECT EXISTS(SELECT 1 FROM study_sessions WHERE id=?1)",
                [request.session_id.to_string()],
                |r| r.get(0),
            )?;
            if exists {
                return Err(conflict());
            }
            let archived: bool = self
                .connection
                .query_row("SELECT archived FROM units WHERE id=?1", [unit_id], |r| {
                    r.get(0)
                })
                .optional()?
                .ok_or_else(invalid)?;
            if archived {
                return Err(invalid());
            }
            let saved = self
                .lesson_read(context, unit_id, Some(composition_revision_id))?
                .ok_or_else(invalid)?;
            if binding_ids.as_ref().is_some_and(|ids| {
                ids.is_empty()
                    || ids.len() > MAX_ITEMS
                    || ids.iter().collect::<std::collections::HashSet<_>>().len() != ids.len()
            }) {
                return Err(invalid());
            }
            let mut items = Vec::new();
            for binding in saved.composition.bindings.iter().filter(|b| {
                b.target_kind == "activity"
                    && binding_ids
                        .as_ref()
                        .is_none_or(|ids| ids.contains(&b.binding_id))
            }) {
                let committed: bool = self.connection.query_row("SELECT EXISTS(SELECT 1 FROM lesson_dependencies WHERE composition_id=?1 AND kind='activity' AND target_id=?2 AND revision_id=?3)",params![saved.id,binding.target_id,binding.revision_id],|r|r.get(0))?;
                if !committed {
                    return Err(invalid());
                }
                let activity = self
                    .lesson_activity(context, unit_id, &binding.revision_id)?
                    .ok_or_else(invalid)?;
                items.push(StudyItem {
                    binding_id: binding.binding_id.clone(),
                    activity,
                });
            }
            if binding_ids
                .as_ref()
                .is_some_and(|ids| ids.len() != items.len())
            {
                return Err(invalid());
            }
            if items.is_empty() || items.len() > MAX_ITEMS {
                return Err(AppError::new(ErrorCode::BudgetExceeded));
            }
            if *mode == StudyMode::Assessment && items.iter().any(instructional) {
                return Err(invalid());
            }
            let draft = initial_draft(items.first());
            StudySession {
                contract_version: 1,
                id: request.session_id,
                unit_id: unit_id.clone(),
                composition_revision_id: saved.id,
                mode: *mode,
                assessment_id: scoped.as_ref().map(|(id, _)| *id),
                binding_ids: binding_ids.clone(),
                phase: *phase,
                started_at_unix_ms: wall_time_ms(),
                revision: 0,
                cursor: 0,
                items,
                draft,
            }
        } else {
            load(&self.connection, request.session_id)?
        };
        if session.revision != request.expected_revision {
            return Err(conflict());
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        match &request.action {
            StudyAction::Start { .. } | StudyAction::StartScopedAssessment { .. } => {}
            StudyAction::CorrectSelfJudgment {
                attempt_id,
                supersedes,
                outcome,
            } => {
                uuid(*attempt_id)?;
                uuid(*supersedes)?;
                let attempt: Option<String> = tx
                    .query_row(
                        "SELECT session_id FROM study_attempts WHERE id=?1",
                        [attempt_id.to_string()],
                        |r| r.get(0),
                    )
                    .optional()?;
                if attempt.as_deref() != Some(&session.id.to_string()) {
                    return Err(invalid());
                }
                let prior: Option<(String, String)> = tx
                    .query_row(
                        "SELECT data,digest FROM study_judgments WHERE id=?1 AND attempt_id=?2",
                        params![supersedes.to_string(), attempt_id.to_string()],
                        |r| Ok((r.get(0)?, r.get(1)?)),
                    )
                    .optional()?;
                let (data, digest) = prior.ok_or_else(invalid)?;
                let prior: AssessmentJudgment = decode(data, digest, 16384)?;
                if prior.source != JudgmentSource::SelfReport {
                    return Err(invalid());
                }
                let mut statement =
                    tx.prepare("SELECT data,digest FROM study_judgments WHERE attempt_id=?1")?;
                let rows = statement
                    .query_map([attempt_id.to_string()], |r| {
                        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
                    })?
                    .collect::<rusqlite::Result<Vec<_>>>()?;
                if rows.len() >= 64 {
                    return Err(AppError::new(ErrorCode::BudgetExceeded));
                }
                for (data, digest) in rows {
                    let judgment: AssessmentJudgment = decode(data, digest, 16384)?;
                    if judgment.supersedes == Some(*supersedes) {
                        return Err(conflict());
                    }
                }
                insert_judgment(
                    &tx,
                    AssessmentJudgment {
                        id: Uuid::new_v4(),
                        attempt_id: *attempt_id,
                        source: JudgmentSource::SelfReport,
                        outcome: *outcome,
                        assessor_version: "learner-v1".into(),
                        supersedes: Some(*supersedes),
                        provenance: prior.provenance.clone(),
                        uncertainty: JudgmentUncertainty::NotEstimated,
                        recorded_at_unix_ms: wall_time_ms(),
                    },
                )?;
            }
            action => {
                let item = session.items.get(session.cursor).ok_or_else(invalid)?;
                match action {
                    StudyAction::SaveResponse {
                        response,
                        external_assistance,
                    } => {
                        validate_response(item, response)?;
                        if session.draft.response.is_none() {
                            session.draft.first_saved_at_unix_ms = wall_time_ms();
                        }
                        session.draft.response = Some(response.clone());
                        session.draft.help.external = *external_assistance;
                    }
                    StudyAction::RevealHint => {
                        if session.mode == StudyMode::Assessment {
                            return Err(invalid());
                        }
                        let available = item.activity.definition["hints"]
                            .as_array()
                            .ok_or_else(corrupt)?
                            .len();
                        if session.draft.help.hints_revealed as usize >= available {
                            return Err(invalid());
                        }
                        session.draft.help.hints_revealed += 1;
                    }
                    StudyAction::RevealSolution => {
                        if session.mode == StudyMode::Assessment {
                            return Err(invalid());
                        }
                        session.draft.help.solution_revealed = true;
                    }
                    StudyAction::Submit {
                        self_rating,
                        recall_rating,
                        self_outcome,
                    } => {
                        if self_rating.is_some_and(|r| !(1..=4).contains(&r)) {
                            return Err(invalid());
                        }
                        let response = session.draft.response.clone().ok_or_else(invalid)?;
                        validate_response(item, &response)?;
                        let outcome = evaluate(item, &response);
                        let attempt = StudyAttempt {
                            id: request.operation_id,
                            session_id: session.id,
                            item_index: session.cursor,
                            activity_revision_id: item.activity.id.clone(),
                            response,
                            help: session.draft.help.clone(),
                            self_rating: *self_rating,
                            recall_rating: *recall_rating,
                            first_saved_at_unix_ms: session.draft.first_saved_at_unix_ms,
                            submitted_at_unix_ms: wall_time_ms(),
                        };
                        let data = encode(&attempt)?;
                        tx.execute(
                            "INSERT INTO study_attempts VALUES(?1,?2,?3,?4,?5)",
                            params![
                                attempt.id.to_string(),
                                session.id.to_string(),
                                session.cursor as i64,
                                data,
                                checksum(data.as_bytes())
                            ],
                        )?;
                        if let Some(rating) = recall_rating {
                            crate::workspace::review::record_review(
                                &tx, &session, &attempt, *rating,
                            )?;
                        }
                        insert_judgment(
                            &tx,
                            AssessmentJudgment {
                                id: Uuid::new_v4(),
                                attempt_id: attempt.id,
                                source: JudgmentSource::Deterministic,
                                outcome,
                                assessor_version: "exact-trim-case-sensitive-v1".into(),
                                supersedes: None,
                                provenance: Some(provenance(
                                    &session,
                                    session.cursor,
                                    AssessorIdentity::Local,
                                )?),
                                uncertainty: JudgmentUncertainty::NotApplicable,
                                recorded_at_unix_ms: attempt.submitted_at_unix_ms,
                            },
                        )?;
                        if let Some(outcome) = self_outcome {
                            insert_judgment(
                                &tx,
                                AssessmentJudgment {
                                    id: Uuid::new_v4(),
                                    attempt_id: attempt.id,
                                    source: JudgmentSource::SelfReport,
                                    outcome: *outcome,
                                    assessor_version: "learner-v1".into(),
                                    supersedes: None,
                                    provenance: Some(provenance(
                                        &session,
                                        session.cursor,
                                        AssessorIdentity::Learner,
                                    )?),
                                    uncertainty: JudgmentUncertainty::NotEstimated,
                                    recorded_at_unix_ms: attempt.submitted_at_unix_ms,
                                },
                            )?;
                        }
                        session.cursor += 1;
                        session.draft = initial_draft(session.items.get(session.cursor));
                    }
                    _ => unreachable!(),
                }
            }
        }
        session.revision = session
            .revision
            .checked_add(1)
            .filter(|r| *r <= crate::workspace::types::MAX_REVISION)
            .ok_or_else(conflict)?;
        validate_session(&session)?;
        let data = encode(&session)?;
        if data.len() > MAX_SESSION {
            return Err(AppError::new(ErrorCode::BudgetExceeded));
        }
        let digest = checksum(data.as_bytes());
        tx.execute("INSERT INTO study_sessions VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(id) DO UPDATE SET revision=excluded.revision,data=excluded.data,digest=excluded.digest",
            params![session.id.to_string(),session.unit_id,session.composition_revision_id,session.revision,data,digest])?;
        tx.execute(
            "INSERT INTO study_operations VALUES(?1,?2,?3,?4,?5)",
            params![
                request.operation_id.to_string(),
                session.id.to_string(),
                hash,
                data,
                digest
            ],
        )?;
        crate::workspace::faults::check(crate::workspace::faults::Phase::BeforeCommit)?;
        operations::durable_commit(tx)?;
        crate::workspace::faults::check(crate::workspace::faults::Phase::AfterCommit)
            .map_err(|_| AppError::new(ErrorCode::OutcomeUnconfirmed))?;
        Ok(session)
    }
}

fn insert_judgment(connection: &Connection, judgment: AssessmentJudgment) -> Result<()> {
    let data = encode(&judgment)?;
    connection.execute(
        "INSERT INTO study_judgments VALUES(?1,?2,?3,?4)",
        params![
            judgment.id.to_string(),
            judgment.attempt_id.to_string(),
            data,
            checksum(data.as_bytes())
        ],
    )?;
    Ok(())
}
fn read_attempts(connection: &Connection, session: &StudySession) -> Result<Vec<StudyAttempt>> {
    let rows = connection.prepare("SELECT id,item_index,data,digest FROM study_attempts WHERE session_id=?1 ORDER BY item_index LIMIT 65")?
        .query_map([session.id.to_string()],|r|Ok((r.get::<_,String>(0)?,r.get::<_,i64>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?)))?.collect::<rusqlite::Result<Vec<_>>>()?;
    if rows.len() != session.cursor {
        return Err(corrupt());
    }
    rows.into_iter()
        .enumerate()
        .map(|(index, (id, item_index, data, digest))| {
            let attempt: StudyAttempt = decode(data, digest, 131072)?;
            let item = session.items.get(index).ok_or_else(corrupt)?;
            if id != attempt.id.to_string()
                || item_index != index as i64
                || attempt.item_index != index
                || attempt.session_id != session.id
                || attempt.activity_revision_id != item.activity.id
                || attempt.self_rating.is_some_and(|r| !(1..=4).contains(&r))
                || attempt.recall_rating.is_some_and(|r| {
                    !(1..=4).contains(&r) || !crate::workspace::review::eligible(session, &attempt)
                })
                || !valid_time(attempt.first_saved_at_unix_ms)
                || !valid_time(attempt.submitted_at_unix_ms)
            {
                return Err(corrupt());
            }
            uuid(attempt.id).map_err(|_| corrupt())?;
            validate_response(item, &attempt.response).map_err(|_| corrupt())?;
            validate_help(session, index, &attempt.help)?;
            Ok(attempt)
        })
        .collect()
}
fn read_judgments(
    connection: &Connection,
    session: &StudySession,
    attempts: &[StudyAttempt],
) -> Result<Vec<AssessmentJudgment>> {
    let rows = connection.prepare("SELECT j.id,j.attempt_id,j.data,j.digest FROM study_judgments j JOIN study_attempts a ON a.id=j.attempt_id WHERE a.session_id=?1 ORDER BY j.rowid LIMIT 4097")?
        .query_map([session.id.to_string()],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?)))?.collect::<rusqlite::Result<Vec<_>>>()?;
    if rows.len() > 4096 {
        return Err(corrupt());
    }
    let mut result: Vec<AssessmentJudgment> = Vec::new();
    let mut superseded = std::collections::HashSet::new();
    for (id, attempt_id, data, digest) in rows {
        let judgment: AssessmentJudgment = decode(data, digest, 16384)?;
        uuid(judgment.id).map_err(|_| corrupt())?;
        let attempt = attempts
            .iter()
            .find(|a| a.id == judgment.attempt_id)
            .ok_or_else(corrupt)?;
        if id != judgment.id.to_string()
            || attempt_id != judgment.attempt_id.to_string()
            || !valid_time(judgment.recorded_at_unix_ms)
        {
            return Err(corrupt());
        }
        if let Some(basis) = &judgment.provenance {
            let assessor = match judgment.source {
                JudgmentSource::Deterministic => AssessorIdentity::Local,
                JudgmentSource::SelfReport => AssessorIdentity::Learner,
                JudgmentSource::Model => return Err(corrupt()),
            };
            if *basis != provenance(session, attempt.item_index, assessor)? {
                return Err(corrupt());
            }
        }
        // No M4 mutation admits model judgments or numerical confidence. M6 must
        // define scoped write/admission policy before activating these contracts.
        if matches!(judgment.uncertainty, JudgmentUncertainty::Reported { .. }) {
            return Err(corrupt());
        }
        match judgment.source {
            JudgmentSource::Model => return Err(corrupt()),
            JudgmentSource::Deterministic => {
                if judgment.supersedes.is_some()
                    || judgment.assessor_version != "exact-trim-case-sensitive-v1"
                    || judgment.outcome
                        != evaluate(&session.items[attempt.item_index], &attempt.response)
                    || result.iter().any(|j| {
                        j.attempt_id == judgment.attempt_id
                            && j.source == JudgmentSource::Deterministic
                    })
                {
                    return Err(corrupt());
                }
            }
            JudgmentSource::SelfReport => {
                if judgment.assessor_version != "learner-v1" {
                    return Err(corrupt());
                }
                if let Some(prior) = judgment.supersedes {
                    if !superseded.insert(prior)
                        || !result.iter().any(|j| {
                            j.id == prior
                                && j.attempt_id == judgment.attempt_id
                                && j.source == JudgmentSource::SelfReport
                        })
                    {
                        return Err(corrupt());
                    }
                } else if result.iter().any(|j| {
                    j.attempt_id == judgment.attempt_id && j.source == JudgmentSource::SelfReport
                }) {
                    return Err(corrupt());
                }
            }
        }
        result.push(judgment);
    }
    for attempt in attempts {
        if !result
            .iter()
            .any(|j| j.attempt_id == attempt.id && j.source == JudgmentSource::Deterministic)
        {
            return Err(corrupt());
        }
    }
    Ok(result)
}

pub(in crate::workspace) fn validate_native_data(connection: &Connection) -> Result<()> {
    let ids = connection
        .prepare("SELECT id,unit_id,composition_id,revision FROM study_sessions")?
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    for (id, unit, composition, revision) in ids {
        let session = load(connection, Uuid::parse_str(&id).map_err(|_| corrupt())?)?;
        if unit != session.unit_id
            || composition != session.composition_revision_id
            || revision != session.revision
        {
            return Err(corrupt());
        }
        let composition_data: String = connection.query_row(
            "SELECT data FROM lesson_revisions WHERE id=?1 AND unit_id=?2",
            params![composition, unit],
            |r| r.get(0),
        )?;
        let composition: crate::workspace::Composition =
            serde_json::from_str(&composition_data).map_err(|_| corrupt())?;
        let bindings: Vec<_> = composition
            .bindings
            .iter()
            .filter(|b| {
                b.target_kind == "activity"
                    && session
                        .binding_ids
                        .as_ref()
                        .is_none_or(|ids| ids.contains(&b.binding_id))
            })
            .collect();
        if bindings.len() != session.items.len() {
            return Err(corrupt());
        }
        for (item, binding) in session.items.iter().zip(bindings) {
            let (definition,digest):(String,String)=connection.query_row("SELECT definition,digest FROM lesson_activity_revisions WHERE id=?1 AND unit_id=?2 AND activity_id=?3",params![binding.revision_id,unit,binding.target_id],|r|Ok((r.get(0)?,r.get(1)?)))?;
            let definition: serde_json::Value = decode(definition, digest, 65536)?;
            if item.binding_id != binding.binding_id
                || item.activity.id != binding.revision_id
                || item.activity.activity_id != binding.target_id
                || item.activity.definition != definition
            {
                return Err(corrupt());
            }
        }
        let attempts = read_attempts(connection, &session)?;
        read_judgments(connection, &session, &attempts)?;
        let operations = connection.prepare("SELECT id,request_hash,result,digest FROM study_operations WHERE session_id=?1 ORDER BY rowid")?
            .query_map([id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?)))?.collect::<rusqlite::Result<Vec<_>>>()?;
        if operations.len() as i64 != session.revision {
            return Err(corrupt());
        }
        let mut previous: Option<StudySession> = None;
        for (index, (id, hash, data, digest)) in operations.into_iter().enumerate() {
            uuid(Uuid::parse_str(&id).map_err(|_| corrupt())?).map_err(|_| corrupt())?;
            if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(corrupt());
            }
            let receipt: StudySession = decode(data, digest, MAX_SESSION)?;
            validate_session(&receipt)?;
            if receipt.id != session.id
                || receipt.unit_id != session.unit_id
                || receipt.items != session.items
                || receipt.composition_revision_id != session.composition_revision_id
                || receipt.assessment_id != session.assessment_id
                || receipt.mode != session.mode
                || receipt.binding_ids != session.binding_ids
                || receipt.phase != session.phase
                || receipt.started_at_unix_ms != session.started_at_unix_ms
                || receipt.revision != index as i64 + 1
                || receipt.cursor > session.cursor
            {
                return Err(corrupt());
            }
            if receipt.revision == session.revision && receipt != session {
                return Err(corrupt());
            }
            let prior_cursor = previous.as_ref().map_or(0, |p| p.cursor);
            if receipt.cursor == prior_cursor + 1 {
                let prior = previous.as_ref().ok_or_else(corrupt)?;
                let attempt = attempts.get(prior_cursor).ok_or_else(corrupt)?;
                if attempt.id.to_string() != id
                    || prior.draft.response.as_ref() != Some(&attempt.response)
                    || prior.draft.help != attempt.help
                    || prior.draft.first_saved_at_unix_ms != attempt.first_saved_at_unix_ms
                {
                    return Err(corrupt());
                }
            } else if receipt.cursor != prior_cursor
                || attempts.iter().any(|a| a.id.to_string() == id)
            {
                return Err(corrupt());
            }
            previous = Some(receipt);
        }
    }
    Ok(())
}
