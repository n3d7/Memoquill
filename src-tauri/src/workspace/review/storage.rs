use super::{algorithm::*, *};
use crate::workspace::{
    DeclaredAssistance, RequestContext, StudyAttempt, StudyMode, StudyPhase, StudySession,
    database::Database, migrations::checksum, operations,
};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Serialize, de::DeserializeOwned};
use std::collections::HashMap;

fn encode(value: &impl Serialize) -> Result<String> {
    serde_json::to_string(value).map_err(|_| invalid())
}
fn decode<T: DeserializeOwned>(data: String, digest: String, limit: usize) -> Result<T> {
    if data.len() > limit || checksum(data.as_bytes()) != digest {
        return Err(corrupt());
    }
    serde_json::from_str(&data).map_err(|_| corrupt())
}
pub(super) fn now_ms() -> Result<i64> {
    let value = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| invalid())?
        .as_millis();
    let value = i64::try_from(value).map_err(|_| invalid())?;
    valid_time(value)?;
    Ok(value)
}
fn journal(connection: &Connection, kind: &str, id: &str) -> Result<()> {
    let next: i64 = connection.query_row(
        "SELECT coalesce(max(position),0)+1 FROM review_journal",
        [],
        |r| r.get(0),
    )?;
    if next > crate::workspace::types::MAX_REVISION {
        return Err(AppError::new(ErrorCode::BudgetExceeded));
    }
    connection.execute(
        "INSERT INTO review_journal VALUES(?1,?2,?3)",
        params![next, kind, id],
    )?;
    Ok(())
}
fn plan(connection: &Connection) -> Result<ReviewPlan> {
    let row: Option<(String, String)> = connection
        .query_row("SELECT data,digest FROM review_plan WHERE id=1", [], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .optional()?;
    let value = match row {
        Some((data, digest)) => decode(data, digest, 65536)?,
        None => ReviewPlan::default(),
    };
    validate_plan(&value).map_err(|_| corrupt())?;
    Ok(value)
}
fn activity(connection: &Connection, unit: &str, revision: &str) -> Result<serde_json::Value> {
    let row: Option<(String, String)> = connection
        .query_row(
            "SELECT definition,digest FROM lesson_activity_revisions WHERE id=?1 AND unit_id=?2",
            params![revision, unit],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let (data, digest) = row.ok_or_else(invalid)?;
    decode(data, digest, 65536)
}
fn reminder_reference(connection: &Connection, unit: &str, revision: &str) -> Result<()> {
    if activity(connection, unit, revision)?["family"] != "procedure" {
        return Err(invalid());
    }
    let committed: bool = connection.query_row("SELECT EXISTS(SELECT 1 FROM lesson_dependencies d JOIN lesson_revisions c ON c.id=d.composition_id WHERE c.unit_id=?1 AND d.kind='activity' AND d.revision_id=?2)",params![unit,revision],|r|r.get(0))?;
    if !committed {
        return Err(invalid());
    }
    Ok(())
}
fn apply_plan(
    connection: &Connection,
    mut value: ReviewPlan,
    request: &ReviewMutation,
    now: i64,
) -> Result<ReviewPlan> {
    if request.operation_id.get_version_num() != 4 {
        return Err(invalid());
    }
    valid_time(now)?;
    if value.revision != request.expected_revision {
        return Err(AppError::new(ErrorCode::RevisionConflict));
    }
    match &request.action {
        ReviewAction::SetPolicy {
            time_zone,
            day_start_hour,
            daily_new_limit,
            daily_review_limit,
            retention_basis_points,
        } => {
            if value.time_zone != *time_zone
                || value.day_start_hour != *day_start_hour
                || value.retention_basis_points != *retention_basis_points
            {
                value.generation = value.generation.checked_add(1).ok_or_else(invalid)?;
            }
            value.time_zone = time_zone.clone();
            value.day_start_hour = *day_start_hour;
            value.daily_new_limit = *daily_new_limit;
            value.daily_review_limit = *daily_review_limit;
            value.retention_basis_points = *retention_basis_points;
        }
        ReviewAction::AddReminder {
            id,
            unit_id,
            activity_revision_id,
            due_day,
            interval_days,
        } => {
            if value.reminders.iter().any(|r| r.id == *id) {
                return Err(invalid());
            }
            reminder_reference(connection, unit_id, activity_revision_id)?;
            value.reminders.push(PracticalReminder {
                id: *id,
                unit_id: unit_id.clone(),
                activity_revision_id: activity_revision_id.clone(),
                due_day: *due_day,
                interval_days: *interval_days,
                archived: false,
                completed_at_unix_ms: None,
            });
        }
        ReviewAction::CompleteReminder { id } => {
            let today = study_day(now, &value.time_zone, value.day_start_hour)?;
            let reminder = value
                .reminders
                .iter_mut()
                .find(|r| r.id == *id && !r.archived)
                .ok_or_else(invalid)?;
            if reminder
                .completed_at_unix_ms
                .is_some_and(|previous| previous > now)
            {
                return Err(AppError::new(ErrorCode::ClockRollback));
            }
            reminder.completed_at_unix_ms = Some(now);
            if let Some(interval) = reminder.interval_days {
                reminder.due_day = today.checked_add(i32::from(interval)).ok_or_else(invalid)?;
            } else {
                reminder.archived = true;
            }
        }
        ReviewAction::ArchiveReminder { id } => {
            value
                .reminders
                .iter_mut()
                .find(|r| r.id == *id)
                .ok_or_else(invalid)?
                .archived = true;
        }
    }
    value.revision = value.revision.checked_add(1).ok_or_else(invalid)?;
    validate_plan(&value)?;
    Ok(value)
}
fn latest(connection: &Connection, revision: &str) -> Result<Option<RecallReviewEvent>> {
    let row: Option<(String,String)> = connection.query_row("SELECT e.data,e.digest FROM recall_states s JOIN recall_review_events e ON e.id=s.event_id WHERE s.activity_revision_id=?1",[revision],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
    row.map(|(data, digest)| decode(data, digest, 8192))
        .transpose()
}
fn daily_counts(connection: &Connection, value: &ReviewPlan, now: i64) -> Result<(u32, u32)> {
    let clock = DayClock::new(&value.time_zone, value.day_start_hour)?;
    let today = clock.day(now)?;
    let mut query = connection.prepare("SELECT data,digest FROM recall_review_events WHERE reviewed_at_ms>=?1 AND reviewed_at_ms<=?2 ORDER BY rowid")?;
    let rows = query.query_map(
        params![
            now.saturating_sub(3 * 86_400_000),
            now.saturating_add(3 * 86_400_000)
        ],
        |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
    )?;
    let (mut new, mut reviewed) = (0u32, 0u32);
    for row in rows {
        let (data, digest) = row?;
        let event: RecallReviewEvent = decode(data, digest, 8192)?;
        if clock.day(event.reviewed_at_unix_ms)? == today {
            if event.prior_event_id.is_none() {
                new += 1;
            } else {
                reviewed += 1;
            }
        }
    }
    Ok((new, reviewed))
}
pub(in crate::workspace) fn eligible(session: &StudySession, attempt: &StudyAttempt) -> bool {
    session.mode == StudyMode::Practice
        && matches!(
            session.phase,
            StudyPhase::Independent | StudyPhase::DelayedCheck
        )
        && session
            .items
            .get(attempt.item_index)
            .is_some_and(|item| item.activity.definition["family"] == "recall")
        && attempt.help.hints_revealed == 0
        && !attempt.help.solution_revealed
        && attempt.help.external == DeclaredAssistance::None
}
pub(in crate::workspace) fn record_review(
    connection: &Connection,
    session: &StudySession,
    attempt: &StudyAttempt,
    rating: u8,
) -> Result<()> {
    if !eligible(session, attempt) || attempt.recall_rating != Some(rating) {
        return Err(invalid());
    }
    let now = attempt.submitted_at_unix_ms.ok_or_else(invalid)?;
    let value = plan(connection)?;
    let prior = latest(connection, &attempt.activity_revision_id)?;
    let last: Option<i64> = connection.query_row(
        "SELECT max(reviewed_at_ms) FROM recall_review_events",
        [],
        |r| r.get(0),
    )?;
    if last.is_some_and(|previous| previous > now) {
        return Err(AppError::new(ErrorCode::ClockRollback));
    }
    let (new, reviewed) = daily_counts(connection, &value, now)?;
    if prior.is_none() && new >= u32::from(value.daily_new_limit)
        || prior.is_some() && reviewed >= u32::from(value.daily_review_limit)
    {
        return Err(AppError::new(ErrorCode::ReviewBudgetExceeded));
    }
    let next = compute(prior.as_ref(), rating, now, &value)?;
    let event = RecallReviewEvent {
        contract_version: 1,
        id: attempt.id,
        attempt_id: attempt.id,
        session_id: session.id,
        unit_id: session.unit_id.clone(),
        activity_revision_id: attempt.activity_revision_id.clone(),
        prior_event_id: prior.as_ref().map(|p| p.id),
        generation: value.generation,
        plan_revision: value.revision,
        algorithm: ALGORITHM.into(),
        parameters: fsrs::DEFAULT_PARAMETERS.to_vec(),
        parameter_sha256: parameter_hash(),
        retention_basis_points: value.retention_basis_points,
        time_zone: value.time_zone.clone(),
        time_zone_rules: rules(),
        day_start_hour: value.day_start_hour,
        reviewed_at_unix_ms: now,
        study_day: study_day(now, &value.time_zone, value.day_start_hour)?,
        rating,
        memory: next.memory,
        raw_interval_days: next.raw_interval_days,
        interval_days: next.interval_days,
        due_day: next.due_day,
    };
    let data = encode(&event)?;
    connection.execute(
        "INSERT INTO recall_review_events VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
        params![
            event.id.to_string(),
            event.attempt_id.to_string(),
            event.activity_revision_id,
            event.prior_event_id.map(|id| id.to_string()),
            now,
            event.study_day,
            data,
            checksum(data.as_bytes())
        ],
    )?;
    connection.execute("INSERT INTO recall_states VALUES(?1,?2) ON CONFLICT(activity_revision_id) DO UPDATE SET event_id=excluded.event_id",params![event.activity_revision_id,event.id.to_string()])?;
    journal(connection, "event", &event.id.to_string())?;
    Ok(())
}
impl Database {
    pub(in crate::workspace) fn review_apply(
        &mut self,
        context: RequestContext,
        request: &ReviewMutation,
    ) -> Result<ReviewPlan> {
        self.review_apply_at(context, request, now_ms()?)
    }
    pub(in crate::workspace) fn review_apply_at(
        &mut self,
        context: RequestContext,
        request: &ReviewMutation,
        now: i64,
    ) -> Result<ReviewPlan> {
        context.authorize()?;
        let request_data = encode(request)?;
        if request_data.len() > 4096 {
            return Err(AppError::new(ErrorCode::BudgetExceeded));
        }
        let hash = checksum(request_data.as_bytes());
        let old: Option<(String, String, String)> = self
            .connection
            .query_row(
                "SELECT request_hash,result,digest FROM review_operations WHERE id=?1",
                [request.operation_id.to_string()],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?;
        if let Some((previous, data, digest)) = old {
            if hash != previous {
                return Err(AppError::new(ErrorCode::RevisionConflict));
            }
            return decode(data, digest, 65536);
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let value = apply_plan(&tx, plan(&tx)?, request, now)?;
        let data = encode(&value)?;
        let digest = checksum(data.as_bytes());
        tx.execute("INSERT INTO review_plan VALUES(1,?1,?2) ON CONFLICT(id) DO UPDATE SET data=excluded.data,digest=excluded.digest",params![data,digest])?;
        tx.execute(
            "INSERT INTO review_operations VALUES(?1,?2,?3,?4,?5,?6)",
            params![
                request.operation_id.to_string(),
                request_data,
                hash,
                now,
                data,
                digest
            ],
        )?;
        journal(&tx, "plan", &request.operation_id.to_string())?;
        crate::workspace::faults::check(crate::workspace::faults::Phase::BeforeCommit)?;
        operations::durable_commit(tx)?;
        crate::workspace::faults::check(crate::workspace::faults::Phase::AfterCommit)
            .map_err(|_| AppError::new(ErrorCode::OutcomeUnconfirmed))?;
        Ok(value)
    }
    pub(in crate::workspace) fn review_overview(
        &self,
        context: RequestContext,
    ) -> Result<ReviewOverview> {
        self.review_overview_at(context, now_ms()?)
    }
    pub(in crate::workspace) fn review_overview_at(
        &self,
        context: RequestContext,
        now: i64,
    ) -> Result<ReviewOverview> {
        context.authorize()?;
        let value = plan(&self.connection)?;
        let today = study_day(now, &value.time_zone, value.day_start_hour)?;
        let (new_used, reviews_used) = daily_counts(&self.connection, &value, now)?;
        let last: Option<i64> = self.connection.query_row(
            "SELECT max(reviewed_at_ms) FROM recall_review_events",
            [],
            |r| r.get(0),
        )?;
        let rollback = last.is_some_and(|last| last > now);
        let units = self.connection.prepare("SELECT h.unit_id,h.revision_id FROM lesson_heads h JOIN units u ON u.id=h.unit_id WHERE u.archived=0 ORDER BY h.unit_id LIMIT 20001")?.query_map([],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?)))?.collect::<rusqlite::Result<Vec<_>>>()?;
        if units.len() > 20000 {
            return Err(AppError::new(ErrorCode::BudgetExceeded));
        }
        let (mut due, mut new) = (Vec::new(), Vec::new());
        let mut seen = std::collections::HashSet::new();
        for (unit, composition) in units {
            let saved = self
                .lesson_read(context, &unit, Some(&composition))?
                .ok_or_else(corrupt)?;
            for binding in saved
                .composition
                .bindings
                .iter()
                .filter(|b| b.target_kind == "activity")
            {
                let d = activity(&self.connection, &unit, &binding.revision_id)?;
                if d["family"] != "recall" || !seen.insert(binding.revision_id.clone()) {
                    continue;
                }
                let state = latest(&self.connection, &binding.revision_id)?;
                let due_day = state.map(|s| s.due_day);
                let prompt = if d["version"] == 2 {
                    d["prompt"]
                        .as_str()
                        .ok_or_else(corrupt)?
                        .replace("{{answer}}", "[blank]")
                } else {
                    d["prompt"].as_str().ok_or_else(corrupt)?.into()
                };
                let item = RecallQueueItem {
                    unit_id: unit.clone(),
                    composition_revision_id: composition.clone(),
                    binding_id: binding.binding_id.clone(),
                    activity_revision_id: binding.revision_id.clone(),
                    prompt,
                    due_day,
                    due_date: due_day.map(day_label).transpose()?,
                };
                if due_day.is_none() {
                    new.push(item);
                } else if due_day.is_some_and(|day| day <= today) {
                    due.push(item);
                }
            }
        }
        due.sort_by_key(|item| (item.due_day, item.unit_id.clone(), item.binding_id.clone()));
        let due_total = due.len() as u32;
        let new_total = new.len() as u32;
        let offered = if rollback {
            Vec::new()
        } else {
            due.into_iter()
                .take(usize::from(
                    value
                        .daily_review_limit
                        .saturating_sub(reviews_used.min(u16::MAX as u32) as u16),
                ))
                .chain(
                    new.into_iter().take(usize::from(
                        value
                            .daily_new_limit
                            .saturating_sub(new_used.min(u16::MAX as u32) as u16),
                    )),
                )
                .take(64)
                .collect()
        };
        Ok(ReviewOverview {
            plan: value,
            now_unix_ms: now,
            today,
            today_date: day_label(today)?,
            time_zone_rules: rules(),
            clock_rollback: rollback,
            new_used,
            reviews_used,
            due_total,
            new_total,
            offered,
        })
    }
}

pub(in crate::workspace) fn validate_native_data(connection: &Connection) -> Result<()> {
    let stored = plan(connection)?;
    let mut values = HashMap::from([(0, ReviewPlan::default())]);
    let operations = connection.prepare("SELECT id,request,request_hash,recorded_at_ms,result,digest FROM review_operations ORDER BY rowid")?.query_map([],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,i64>(3)?,r.get::<_,String>(4)?,r.get::<_,String>(5)?)))?.collect::<rusqlite::Result<Vec<_>>>()?;
    let mut operation_ids = HashMap::new();
    let mut current = ReviewPlan::default();
    for (id, request, hash, now, data, digest) in operations {
        if request.len() > 4096 || checksum(request.as_bytes()) != hash {
            return Err(corrupt());
        }
        let request: ReviewMutation = serde_json::from_str(&request).map_err(|_| corrupt())?;
        if id != request.operation_id.to_string() {
            return Err(corrupt());
        }
        let receipt: ReviewPlan = decode(data, digest, 65536)?;
        let computed = apply_plan(connection, current, &request, now).map_err(|_| corrupt())?;
        if computed != receipt {
            return Err(corrupt());
        }
        operation_ids.insert(id, receipt.clone());
        values.insert(receipt.revision, receipt.clone());
        current = receipt;
    }
    if current != stored {
        return Err(corrupt());
    }
    let mut latest: HashMap<String, RecallReviewEvent> = HashMap::new();
    let rows = connection.prepare("SELECT id,attempt_id,activity_revision_id,prior_event_id,reviewed_at_ms,study_day,data,digest FROM recall_review_events ORDER BY rowid")?.query_map([],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,Option<String>>(3)?,r.get::<_,i64>(4)?,r.get::<_,i32>(5)?,r.get::<_,String>(6)?,r.get::<_,String>(7)?)))?.collect::<rusqlite::Result<Vec<_>>>()?;
    let mut event_ids = std::collections::HashSet::new();
    let mut events = HashMap::new();
    let mut event_order = Vec::new();
    let mut last_time = None;
    for (id, attempt_id, revision, previous, now, day, data, digest) in rows {
        let event: RecallReviewEvent = decode(data, digest, 8192)?;
        let value = values.get(&event.plan_revision).ok_or_else(corrupt)?;
        let prior = latest.get(&revision);
        let computed = compute(prior, event.rating, now, value).map_err(|_| corrupt())?;
        let (attempt_data, attempt_digest): (String, String) = connection.query_row(
            "SELECT data,digest FROM study_attempts WHERE id=?1",
            [&attempt_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let attempt: StudyAttempt = decode(attempt_data, attempt_digest, 131072)?;
        let (session_data, session_digest): (String, String) = connection.query_row(
            "SELECT data,digest FROM study_sessions WHERE id=?1",
            [attempt.session_id.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let session: StudySession = decode(session_data, session_digest, 1048576)?;
        if event.contract_version != 1
            || event.id.to_string() != id
            || event.id != event.attempt_id
            || event.attempt_id.to_string() != attempt_id
            || event.activity_revision_id != revision
            || event.prior_event_id.map(|p| p.to_string()) != previous
            || event.prior_event_id != prior.map(|p| p.id)
            || event.reviewed_at_unix_ms != now
            || event.study_day != day
            || day != study_day(now, &value.time_zone, value.day_start_hour)?
            || event.algorithm != ALGORITHM
            || event.parameters != fsrs::DEFAULT_PARAMETERS
            || event.parameter_sha256 != parameter_hash()
            || event.generation != value.generation
            || event.time_zone != value.time_zone
            || event.day_start_hour != value.day_start_hour
            || event.time_zone_rules != rules()
            || event.retention_basis_points != value.retention_basis_points
            || event.memory != computed.memory
            || event.raw_interval_days != computed.raw_interval_days
            || event.interval_days != computed.interval_days
            || event.due_day != computed.due_day
            || !eligible(&session, &attempt)
            || attempt.recall_rating != Some(event.rating)
            || attempt.submitted_at_unix_ms != Some(now)
            || event.session_id != session.id
            || event.unit_id != session.unit_id
            || event.activity_revision_id != attempt.activity_revision_id
            || last_time.is_some_and(|last| last > now)
        {
            return Err(corrupt());
        }
        event_ids.insert(event.attempt_id);
        last_time = Some(now);
        event_order.push(id.clone());
        events.insert(id, event.clone());
        latest.insert(revision, event);
    }
    let attempts = connection
        .prepare("SELECT data,digest FROM study_attempts")?
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    for (data, digest) in attempts {
        let attempt: StudyAttempt = decode(data, digest, 131072)?;
        if attempt.recall_rating.is_some() != event_ids.contains(&attempt.id) {
            return Err(corrupt());
        }
    }
    let timeline = connection
        .prepare("SELECT position,kind,record_id FROM review_journal ORDER BY position")?
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    if timeline.len() != operation_ids.len() + events.len() {
        return Err(corrupt());
    }
    let mut active = ReviewPlan::default();
    let mut clock = DayClock::new(&active.time_zone, active.day_start_hour)?;
    let mut recent = std::collections::VecDeque::<(i64, bool)>::new();
    let mut visited = std::collections::HashSet::new();
    let mut event_index = 0;
    for (index, (position, kind, id)) in timeline.into_iter().enumerate() {
        if position != index as i64 + 1 || !visited.insert((kind.clone(), id.clone())) {
            return Err(corrupt());
        }
        match kind.as_str() {
            "plan" => {
                let next = operation_ids.get(&id).ok_or_else(corrupt)?;
                if next.revision != active.revision + 1 {
                    return Err(corrupt());
                }
                active = next.clone();
                clock = DayClock::new(&active.time_zone, active.day_start_hour)?;
            }
            "event" => {
                if event_order.get(event_index) != Some(&id) {
                    return Err(corrupt());
                }
                event_index += 1;
                let event = events.get(&id).ok_or_else(corrupt)?;
                if event.plan_revision != active.revision {
                    return Err(corrupt());
                }
                let lower = event.reviewed_at_unix_ms.saturating_sub(3 * 86_400_000);
                while recent.front().is_some_and(|(time, _)| *time < lower) {
                    recent.pop_front();
                }
                let today = clock.day(event.reviewed_at_unix_ms)?;
                let (mut new, mut reviewed) = (0u32, 0u32);
                for (time, was_new) in &recent {
                    if clock.day(*time)? == today {
                        if *was_new {
                            new += 1;
                        } else {
                            reviewed += 1;
                        }
                    }
                }
                let is_new = event.prior_event_id.is_none();
                if is_new && new >= u32::from(active.daily_new_limit)
                    || !is_new && reviewed >= u32::from(active.daily_review_limit)
                {
                    return Err(corrupt());
                }
                recent.push_back((event.reviewed_at_unix_ms, is_new));
                if recent.len() > 10000 {
                    return Err(corrupt());
                }
            }
            _ => return Err(corrupt()),
        }
    }
    let states = connection
        .prepare("SELECT activity_revision_id,event_id FROM recall_states")?
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    if states.len() != latest.len() {
        return Err(corrupt());
    }
    for (revision, id) in states {
        if latest.get(&revision).is_none_or(|e| e.id.to_string() != id) {
            return Err(corrupt());
        }
    }
    Ok(())
}
