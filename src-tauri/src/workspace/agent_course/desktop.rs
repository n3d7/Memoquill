use super::*;
use crate::{
    organization::{Organization, Snapshot},
    workspace::{
        RequestContext, WorkspaceId,
        content::{documents, domain_error, organization_store},
        database::Database,
        mcp_drafts::DraftActor,
        mcp_reads::{StructureKind, structure},
    },
};
use rusqlite::TransactionBehavior;
use serde_json::{Value, json};

pub(crate) const PAGE_SIZE: usize = 24;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CourseDraftRequest {
    owner_key: Uuid,
    action: DraftAction,
}

#[derive(Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum DraftAction {
    Create {
        draft_id: Uuid,
        title: String,
        base_revision: u64,
    },
    Fork {
        from_id: Uuid,
        from_revision: u64,
        draft_id: Uuid,
        title: String,
        base_revision: u64,
        keep_chunks: u32,
    },
    List,
    Read {
        draft_id: Uuid,
        revision: Option<u64>,
        offset: u32,
    },
    Structure {
        draft_id: Uuid,
        revision: u64,
        category: StructureKind,
        offset: u32,
    },
    Append {
        draft_id: Uuid,
        revision: u64,
        operation_id: Uuid,
        operations: Vec<CourseOperation>,
    },
    Validate {
        draft_id: Uuid,
        revision: u64,
    },
    Submit {
        draft_id: Uuid,
        revision: u64,
        operation_id: Uuid,
    },
    ApplyDelegated {
        draft_id: Uuid,
        revision: u64,
        operation_id: Uuid,
        hash: String,
    },
    Discard {
        draft_id: Uuid,
        revision: u64,
        operation_id: Uuid,
    },
}

impl CourseDraftRequest {
    pub(in crate::workspace) fn is_mutation(&self) -> bool {
        matches!(
            self.action,
            DraftAction::Create { .. }
                | DraftAction::Fork { .. }
                | DraftAction::Append { .. }
                | DraftAction::Submit { .. }
                | DraftAction::Discard { .. }
                | DraftAction::ApplyDelegated { .. }
        )
    }
}

impl Database {
    pub(in crate::workspace) fn mcp_course_draft(
        &mut self,
        request: CourseDraftRequest,
        delegation: Option<&Delegation>,
    ) -> Result<Value> {
        let actor = DraftActor::from_key(&self.id, request.owner_key)?;
        let actor = actor.as_str();
        if request.is_mutation() {
            self.authorize_workspace(RequestContext::local_ui(), &self.id)?;
        }
        match request.action {
            DraftAction::List => Ok(json!({"drafts": list(&self.connection, actor)?})),
            DraftAction::Read {
                draft_id,
                revision,
                offset,
            } => {
                let record = load(&self.connection, draft_id, actor)?.ok_or_else(denied)?;
                if offset != 0 && revision.is_none() {
                    return Err(conflict());
                }
                if let Some(revision) = revision {
                    verify_revision(&record, revision)?;
                }
                let offset = offset as usize;
                let mut operations = record
                    .chunks
                    .iter()
                    .flat_map(|chunk| &chunk.operations)
                    .skip(offset);
                let mut page = Vec::new();
                let mut size = 0;
                for operation in operations.by_ref().take(PAGE_SIZE) {
                    let bytes = serde_json::to_vec(operation).map_err(|_| corrupt())?.len();
                    if !page.is_empty() && size + bytes > 512 * 1024 {
                        return Ok(
                            json!({"draft": summary(&record), "operations": page, "nextOffset": offset + page.len()}),
                        );
                    }
                    size += bytes;
                    page.push(operation);
                }
                let next = operations.next().map(|_| offset + page.len());
                Ok(json!({"draft": summary(&record), "operations": page, "nextOffset": next}))
            }
            DraftAction::Structure {
                draft_id,
                revision,
                category,
                offset,
            } => {
                let record = load(&self.connection, draft_id, actor)?.ok_or_else(denied)?;
                verify_revision(&record, revision)?;
                let overlay =
                    simulate(&self.connection, &self.id, &record).map_err(|error| error.error)?;
                let (items, next) = structure(&overlay, category, offset);
                Ok(json!({"draft": summary(&record), "items": items, "nextOffset": next}))
            }
            DraftAction::Validate { draft_id, revision } => {
                let record = load(&self.connection, draft_id, actor)?.ok_or_else(denied)?;
                verify_revision(&record, revision)?;
                match simulate(&self.connection, &self.id, &record) {
                    Ok(_) => Ok(
                        json!({"valid": true, "draft": summary(&record), "hash": proposal_hash(&record)?}),
                    ),
                    Err(error) => Ok(validation_error(error)),
                }
            }
            DraftAction::Create {
                draft_id,
                title,
                base_revision,
            } => {
                if draft_id.get_version_num() != 4 || title.len() > 256 || title.trim().is_empty() {
                    return Err(invalid());
                }
                let tx = self
                    .connection
                    .transaction_with_behavior(TransactionBehavior::Immediate)?;
                if let Some(record) = load(&tx, draft_id, actor)? {
                    if record.title != title || record.base_revision != base_revision {
                        return Err(idempotency());
                    }
                    return Ok(summary(&record));
                }
                if organization_store::load_connection(&tx, &self.id)?.revision != base_revision {
                    return Err(conflict());
                }
                ensure_capacity(&tx)?;
                let record = Record {
                    version: 1,
                    id: draft_id,
                    title,
                    base_revision,
                    chunks: Vec::new(),
                    status: Status::Editing,
                    transition_id: None,
                    discard_id: None,
                    commit: None,
                    undo_id: None,
                };
                persist(&tx, actor, &record)?;
                super::super::operations::durable_commit(tx)?;
                Ok(summary(&record))
            }
            DraftAction::Fork {
                from_id,
                from_revision,
                draft_id,
                title,
                base_revision,
                keep_chunks,
            } => {
                if draft_id.get_version_num() != 4
                    || draft_id == from_id
                    || title.trim().is_empty()
                    || title.len() > 256
                {
                    return Err(invalid());
                }
                let tx = self
                    .connection
                    .transaction_with_behavior(TransactionBehavior::Immediate)?;
                let mut fork = load(&tx, from_id, actor)?.ok_or_else(denied)?;
                verify_revision(&fork, from_revision)?;
                if keep_chunks as usize > fork.chunks.len() {
                    return Err(invalid());
                }
                fork.chunks.truncate(keep_chunks as usize);
                fork.id = draft_id;
                fork.title = title;
                fork.base_revision = base_revision;
                fork.status = Status::Editing;
                fork.transition_id = None;
                fork.discard_id = None;
                fork.commit = None;
                fork.undo_id = None;
                if let Some(existing) = load(&tx, draft_id, actor)? {
                    if existing.title != fork.title
                        || proposal_hash(&existing)? != proposal_hash(&fork)?
                    {
                        return Err(idempotency());
                    }
                    return Ok(summary(&existing));
                }
                if let Err(error) = simulate(&tx, &self.id, &fork) {
                    return Ok(validation_error(error));
                }
                ensure_capacity(&tx)?;
                persist(&tx, actor, &fork)?;
                super::super::operations::durable_commit(tx)?;
                Ok(summary(&fork))
            }
            DraftAction::Append {
                draft_id,
                revision,
                operation_id,
                operations,
            } => {
                let tx = self
                    .connection
                    .transaction_with_behavior(TransactionBehavior::Immediate)?;
                let mut record = load(&tx, draft_id, actor)?.ok_or_else(denied)?;
                if operation_id.get_version_num() != 4
                    || operations.is_empty()
                    || operations.len() > MAX_CHUNK_OPERATIONS
                {
                    return Err(invalid());
                }
                if serde_json::to_vec(&operations)
                    .map_err(|_| invalid())?
                    .len()
                    > 64 * 1024
                {
                    return Err(budget());
                }
                let hash = payload_hash(&operations)?;
                if let Some((index, chunk)) = record
                    .chunks
                    .iter()
                    .enumerate()
                    .find(|(_, entry)| entry.operation_id == operation_id)
                {
                    if chunk.hash != hash || revision != index as u64 + 1 {
                        return Err(idempotency());
                    }
                    return Ok(chunk_result(&record, index));
                }
                verify_revision(&record, revision)?;
                if record.status != Status::Editing || record.transition_id == Some(operation_id) {
                    return Err(conflict());
                }
                if record.chunks.len() >= MAX_CHUNKS
                    || record.operation_count() + operations.len() > MAX_GROUP_OPERATIONS
                {
                    return Err(budget());
                }
                record.chunks.push(Chunk {
                    operation_id,
                    hash,
                    operations,
                });
                if let Err(error) = simulate(&tx, &self.id, &record) {
                    return Ok(validation_error(error));
                }
                persist(&tx, actor, &record)?;
                super::super::operations::durable_commit(tx)?;
                Ok(chunk_result(&record, record.chunks.len() - 1))
            }
            DraftAction::Submit {
                draft_id,
                revision,
                operation_id,
            } => self.transition_course_draft(
                actor,
                draft_id,
                revision,
                operation_id,
                Status::Submitted,
            ),
            DraftAction::ApplyDelegated {
                draft_id,
                revision,
                operation_id,
                hash,
            } => {
                let tx = self
                    .connection
                    .transaction_with_behavior(TransactionBehavior::Immediate)?;
                let mut record = load(&tx, draft_id, actor)?.ok_or_else(denied)?;
                if let Some(receipt) = &record.commit {
                    if receipt.delegated
                        && same_commit(&record, receipt, revision, &hash, operation_id)
                    {
                        return Ok(summary(&record));
                    }
                    return Err(conflict());
                }
                let Some(lease) = delegation else {
                    return Err(approval_required());
                };
                let mut policy = lease.lock()?;
                policy.authorize()?;
                verify_submission(&record, revision, &hash, operation_id)?;
                if !super::delegation::allows(&record) {
                    return Err(approval_required());
                }
                // Reserve only after all storage validation and before the
                // single commit. An uncertain outcome never replenishes quota.
                apply_group(&tx, &self.id, &mut record, operation_id, hash, true)?;
                persist(&tx, actor, &record)?;
                policy.reserve(record.operation_count())?;
                super::super::operations::durable_commit(tx)?;
                Ok(summary(&record))
            }
            DraftAction::Discard {
                draft_id,
                revision,
                operation_id,
            } => self.transition_course_draft(
                actor,
                draft_id,
                revision,
                operation_id,
                Status::Discarded,
            ),
        }
    }

    fn transition_course_draft(
        &mut self,
        actor: &str,
        id: Uuid,
        revision: u64,
        operation_id: Uuid,
        status: Status,
    ) -> Result<Value> {
        if operation_id.get_version_num() != 4 {
            return Err(invalid());
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut record = load(&tx, id, actor)?.ok_or_else(denied)?;
        let prior = if status == Status::Discarded {
            record.discard_id
        } else {
            record.transition_id
        };
        if prior == Some(operation_id) {
            if record.status != status || record.revision() != revision.saturating_add(1) {
                return Err(idempotency());
            }
            return Ok(summary(&record));
        }
        verify_revision(&record, revision)?;
        if record
            .chunks
            .iter()
            .any(|chunk| chunk.operation_id == operation_id)
        {
            return Err(idempotency());
        }
        if status == Status::Submitted {
            if record.status != Status::Editing || record.chunks.is_empty() {
                return Err(invalid());
            }
            if let Err(error) = simulate(&tx, &self.id, &record) {
                return Ok(validation_error(error));
            }
            record.transition_id = Some(operation_id);
        } else {
            if !matches!(record.status, Status::Editing | Status::Submitted)
                || record.transition_id == Some(operation_id)
            {
                return Err(conflict());
            }
            record.discard_id = Some(operation_id);
        }
        record.status = status;
        persist(&tx, actor, &record)?;
        super::super::operations::durable_commit(tx)?;
        Ok(summary(&record))
    }
}

#[derive(Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub(crate) enum CourseReviewRequest {
    List,
    Inspect {
        draft_id: Uuid,
        revision: u64,
        offset: u32,
    },
    Undo {
        draft_id: Uuid,
        revision: u64,
        operation_id: Uuid,
    },
    Apply {
        draft_id: Uuid,
        revision: u64,
        hash: String,
        operation_id: Uuid,
    },
}

impl CourseReviewRequest {
    pub(in crate::workspace) fn is_mutation(&self) -> bool {
        matches!(self, Self::Apply { .. } | Self::Undo { .. })
    }
}

impl Database {
    pub(in crate::workspace) fn review_course_draft(
        &mut self,
        context: RequestContext,
        request: CourseReviewRequest,
    ) -> Result<Value> {
        context.authorize()?;
        match request {
            CourseReviewRequest::List => {
                let mut query = self.connection.prepare(
                    "SELECT id,actor,data FROM agent_course_drafts
                     WHERE json_extract(data,'$.status') IN ('submitted','applied','undone') ORDER BY id LIMIT 65"
                )?;
                let mut rows = query.query([])?;
                let mut drafts = Vec::new();
                while let Some(row) = rows.next()? {
                    if drafts.len() >= MAX_DRAFTS as usize {
                        return Err(corrupt());
                    }
                    drafts.push(summary(&reviewed_record(row)?));
                }
                Ok(json!({"drafts": drafts}))
            }
            CourseReviewRequest::Inspect {
                draft_id,
                revision,
                offset,
            } => {
                let (_, record) = load_for_review(&self.connection, draft_id)?;
                verify_revision(&record, revision)?;
                let validation = match simulate(&self.connection, &self.id, &record) {
                    Ok(_) => json!({"valid": true}),
                    Err(error) => validation_error(error),
                };
                let offset = offset as usize;
                let mut operations = record
                    .chunks
                    .iter()
                    .flat_map(|chunk| &chunk.operations)
                    .skip(offset);
                let page: Vec<_> = operations.by_ref().take(8).collect();
                let next = operations.next().map(|_| offset + page.len());
                Ok(
                    json!({"draft": summary(&record), "operations": page, "nextOffset": next,
                    "validation": validation, "hash": proposal_hash(&record)?,
                    "undoAvailable": super::undo::preview(&self.connection, &self.id, &record).is_ok()}),
                )
            }
            CourseReviewRequest::Undo {
                draft_id,
                revision,
                operation_id,
            } => {
                self.authorize_workspace(context, &self.id)?;
                let tx = self
                    .connection
                    .transaction_with_behavior(TransactionBehavior::Immediate)?;
                let (actor, mut record) = load_for_review(&tx, draft_id)?;
                if record.undo_id == Some(operation_id)
                    && record.revision() == revision.saturating_add(1)
                {
                    return Ok(summary(&record));
                }
                verify_revision(&record, revision)?;
                if operation_id.get_version_num() != 4 || Some(operation_id) == record.transition_id
                {
                    return Err(invalid());
                }
                let snapshot = super::undo::preview(&tx, &self.id, &record)?;
                organization_store::save(&tx, &snapshot)?;
                record.status = Status::Undone;
                record.undo_id = Some(operation_id);
                persist(&tx, &actor, &record)?;
                super::super::operations::durable_commit(tx)?;
                Ok(summary(&record))
            }
            CourseReviewRequest::Apply {
                draft_id,
                revision,
                hash,
                operation_id,
            } => {
                self.authorize_workspace(context, &self.id)?;
                let tx = self
                    .connection
                    .transaction_with_behavior(TransactionBehavior::Immediate)?;
                let (actor, mut record) = load_for_review(&tx, draft_id)?;
                if let Some(receipt) = &record.commit {
                    if !receipt.delegated
                        && same_commit(&record, receipt, revision, &hash, operation_id)
                    {
                        return Ok(summary(&record));
                    }
                    return Err(conflict());
                }
                verify_submission(&record, revision, &hash, operation_id)?;
                apply_group(&tx, &self.id, &mut record, operation_id, hash, false)?;
                persist(&tx, &actor, &record)?;
                super::super::operations::durable_commit(tx)?;
                Ok(summary(&record))
            }
        }
    }
}

fn same_commit(
    record: &Record,
    receipt: &CommitReceipt,
    revision: u64,
    hash: &str,
    operation_id: Uuid,
) -> bool {
    receipt.operation_id == operation_id
        && receipt.hash == hash
        && 2 + record.chunks.len() as u64 == revision
}

fn approval_required() -> AppError {
    AppError::new(ErrorCode::ApprovalRequired)
}

fn verify_submission(record: &Record, revision: u64, hash: &str, operation_id: Uuid) -> Result<()> {
    verify_revision(record, revision)?;
    if record.status != Status::Submitted
        || hash != proposal_hash(record)?
        || operation_id.get_version_num() != 4
        || Some(operation_id) == record.transition_id
    {
        return Err(conflict());
    }
    Ok(())
}

fn reviewed_record(row: &rusqlite::Row<'_>) -> Result<Record> {
    let id: String = row.get(0)?;
    let id = Uuid::parse_str(&id).map_err(|_| corrupt())?;
    let actor: String = row.get(1)?;
    let data: String = row.get(2)?;
    let record: Record = decode(&data)?;
    record.validate(id, &actor)?;
    Ok(record)
}

fn load_for_review(connection: &Connection, id: Uuid) -> Result<(String, Record)> {
    let mut query = connection.prepare(
        "SELECT id,actor,data FROM agent_course_drafts WHERE id=?1 AND json_extract(data,'$.status') IN ('submitted','applied','undone')"
    )?;
    let mut rows = query.query([id.to_string()])?;
    let row = rows.next()?.ok_or_else(denied)?;
    let actor = row.get(1)?;
    Ok((actor, reviewed_record(row)?))
}

fn apply_group(
    tx: &Connection,
    workspace: &WorkspaceId,
    record: &mut Record,
    operation_id: Uuid,
    hash: String,
    delegated: bool,
) -> Result<()> {
    let overlay = simulate(tx, workspace, record).map_err(|error| error.error)?;
    organization_store::save(tx, &overlay)?;
    let mut documents = Vec::new();
    for operation in record.chunks.iter().flat_map(|chunk| &chunk.operations) {
        if let CourseOperation::Document {
            resource_id,
            expected_revision,
            document,
        } = operation
        {
            let value = documents::save_revision(
                tx,
                &crate::workspace::SaveDocument {
                    contract_version: 1,
                    workspace_id: workspace.clone(),
                    operation_id: crate::workspace::OperationId::try_from(
                        operation_id.to_string(),
                    )?,
                    resource_id: *resource_id,
                    expected_revision: *expected_revision,
                    document: document.clone(),
                },
            )?;
            tx.execute(
                "INSERT INTO agent_authored_documents(revision_id,proposal_id) VALUES(?1,?2)",
                params![value.id.to_string(), record.id.to_string()],
            )?;
            documents.push(DocumentReceipt {
                resource_id: *resource_id,
                revision_id: value.id,
            });
        }
    }
    record.commit = Some(CommitReceipt {
        operation_id,
        hash,
        result_revision: overlay.revision,
        documents,
        delegated,
    });
    record.status = Status::Applied;
    Ok(())
}

struct SimulationError {
    index: Option<usize>,
    error: AppError,
}

fn simulate(
    connection: &Connection,
    workspace: &WorkspaceId,
    record: &Record,
) -> std::result::Result<Snapshot, SimulationError> {
    let fail = |error| SimulationError { index: None, error };
    let base = organization_store::load_connection(connection, workspace).map_err(fail)?;
    if base.revision != record.base_revision {
        return Err(fail(conflict()));
    }
    let mut overlay =
        Organization::from_snapshot(base).map_err(|error| fail(domain_error(error)))?;
    let mut heads = std::collections::HashMap::new();
    for (index, operation) in record
        .chunks
        .iter()
        .flat_map(|chunk| &chunk.operations)
        .enumerate()
    {
        let apply = || -> Result<()> {
            match operation {
                CourseOperation::Organization { command } => overlay
                    .apply(
                        overlay.snapshot().workspace_id,
                        overlay.revision(),
                        command.clone(),
                    )
                    .map_err(domain_error),
                CourseOperation::Document {
                    resource_id,
                    expected_revision,
                    ..
                } => {
                    validate_operation(operation)?;
                    if !overlay
                        .snapshot()
                        .resources
                        .get(resource_id)
                        .is_some_and(|row| !row.archived)
                    {
                        return Err(invalid());
                    }
                    let revision = match heads.get(resource_id) {
                        Some(revision) => *revision,
                        None => documents::read(connection, *resource_id, None)?
                            .map_or(0, |row| row.revision),
                    };
                    if revision != *expected_revision {
                        return Err(conflict());
                    }
                    heads.insert(*resource_id, revision + 1);
                    Ok(())
                }
            }
        };
        let mut apply = apply;
        apply().map_err(|error| SimulationError {
            index: Some(index),
            error,
        })?;
    }
    Ok(overlay.into())
}

fn chunk_result(record: &Record, index: usize) -> Value {
    json!({"draftId": record.id, "revision": index as u64 + 2, "completedChunks": index + 1})
}

fn validation_error(error: SimulationError) -> Value {
    json!({"valid": false, "operationIndex": error.index, "code": error.error.code})
}

fn verify_revision(record: &Record, revision: u64) -> Result<()> {
    if record.revision() != revision || record.status == Status::Discarded {
        return Err(conflict());
    }
    Ok(())
}

fn load(connection: &Connection, id: Uuid, actor: &str) -> Result<Option<Record>> {
    let data: Option<String> = connection
        .query_row(
            "SELECT data FROM agent_course_drafts WHERE id=?1 AND actor=?2",
            params![id.to_string(), actor],
            |row| row.get(0),
        )
        .optional()?;
    data.map(|data| {
        let record: Record = decode(&data)?;
        record.validate(id, actor)?;
        Ok(record)
    })
    .transpose()
}

fn ensure_capacity(tx: &Connection) -> Result<()> {
    let count: i64 = tx.query_row("SELECT count(*) FROM agent_course_drafts", [], |row| {
        row.get(0)
    })?;
    if count >= MAX_DRAFTS {
        // Completed discards retain retry identity until the next capacity cleanup.
        tx.execute(
            "DELETE FROM agent_course_drafts WHERE json_extract(data,'$.status')='discarded'",
            [],
        )?;
        let remaining: i64 =
            tx.query_row("SELECT count(*) FROM agent_course_drafts", [], |row| {
                row.get(0)
            })?;
        if remaining >= MAX_DRAFTS {
            return Err(budget());
        }
    }
    Ok(())
}

fn persist(connection: &Connection, actor: &str, record: &Record) -> Result<()> {
    record.validate(record.id, actor)?;
    let data = serde_json::to_string(record).map_err(|_| invalid())?;
    if data.len() > MAX_GROUP_BYTES {
        return Err(budget());
    }
    let written = connection.execute(
        "INSERT INTO agent_course_drafts(id,actor,data) VALUES (?1,?2,?3)
         ON CONFLICT(id) DO UPDATE SET data=excluded.data WHERE agent_course_drafts.actor=excluded.actor",
        params![record.id.to_string(), actor, data],
    )?;
    if written != 1 {
        return Err(denied());
    }
    Ok(())
}

fn list(connection: &Connection, actor: &str) -> Result<Vec<Value>> {
    let mut query = connection.prepare(
        "SELECT id,data FROM agent_course_drafts WHERE actor=?1 AND json_extract(data,'$.status')!='discarded' ORDER BY id LIMIT 65"
    )?;
    let rows = query.query_map([actor], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut result = Vec::new();
    for row in rows {
        if result.len() >= MAX_DRAFTS as usize {
            return Err(corrupt());
        }
        let (id, data) = row?;
        let id = Uuid::parse_str(&id).map_err(|_| corrupt())?;
        let record: Record = decode(&data)?;
        record.validate(id, actor)?;
        result.push(summary(&record));
    }
    Ok(result)
}

fn summary(record: &Record) -> Value {
    json!({"id": record.id, "title": record.title, "revision": record.revision(),
        "baseRevision": record.base_revision, "status": record.status,
        "completedChunks": record.chunks.len(), "operations": record.operation_count(), "receipt": record.commit})
}

fn invalid() -> AppError {
    AppError::new(ErrorCode::InvalidRequest)
}
fn denied() -> AppError {
    AppError::new(ErrorCode::PermissionDenied)
}
fn budget() -> AppError {
    AppError::new(ErrorCode::BudgetExceeded)
}
fn conflict() -> AppError {
    AppError::new(ErrorCode::RevisionConflict)
}
fn idempotency() -> AppError {
    AppError::new(ErrorCode::IdempotencyConflict)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    struct Fixture {
        temp: tempfile::TempDir,
        database: Database,
        key: Uuid,
    }

    impl Fixture {
        fn new() -> Self {
            let temp = tempfile::tempdir().unwrap();
            let database = Database::open(&temp.path().join("workspace")).unwrap();
            Self {
                temp,
                database,
                key: Uuid::new_v4(),
            }
        }

        fn draft(&mut self, action: Value, lease: Option<&Delegation>) -> Result<Value> {
            let request =
                serde_json::from_value(json!({"ownerKey": self.key, "action": action})).unwrap();
            self.database.mcp_course_draft(request, lease)
        }

        fn review(&mut self, action: Value) -> Result<Value> {
            self.database.review_course_draft(
                RequestContext::local_ui(),
                serde_json::from_value(action).unwrap(),
            )
        }

        fn summary(&self) -> Snapshot {
            self.database.library(RequestContext::local_ui()).unwrap()
        }

        fn create(&mut self, title: &str) -> Uuid {
            let id = Uuid::new_v4();
            self.draft(json!({"kind":"create","draftId":id,"title":title,"baseRevision":self.summary().revision}), None).unwrap();
            id
        }

        fn submit(&mut self, draft: Uuid, operations: Vec<Value>) -> Value {
            self.draft(
                json!({"kind":"append","draftId":draft,"revision":1,
                "operationId":Uuid::new_v4(),"operations":operations}),
                None,
            )
            .unwrap();
            let submitted = self
                .draft(
                    json!({"kind":"submit","draftId":draft,"revision":2,
                "operationId":Uuid::new_v4()}),
                    None,
                )
                .unwrap();
            assert_eq!(submitted["status"], "submitted");
            self.draft(
                json!({"kind":"validate","draftId":draft,"revision":3}),
                None,
            )
            .unwrap()
        }
    }

    fn subject(id: Uuid, title: &str) -> Value {
        json!({"type":"organization","command":{"type":"createSubject","id":id,"title":title}})
    }

    #[test]
    fn mcp_course_batches_resume_and_commit_only_exact_reviewed_changes() {
        let mut f = Fixture::new();
        let group = f.create("Two-module course");
        let source = Uuid::new_v4();
        let resource = Uuid::new_v4();
        let unit = Uuid::new_v4();
        let chunk = json!({"kind":"append","draftId":group,"revision":1,"operationId":Uuid::new_v4(),
        "operations":[subject(source,"Literature"),
            {"type":"organization","command":{"type":"createResource","id":resource,"title":"Reference text"}},
            {"type":"document","resourceId":resource,"expectedRevision":0,"document":{
                "schemaVersion":1,"blocks":[{"id":Uuid::new_v4(),"content":{"type":"code","text":"Synthetic text"}}]
            }}
        ]});
        let first = f.draft(chunk.clone(), None).unwrap();
        assert_eq!(first["revision"], 2, "{first}");
        let resumed = Database::open(&f.temp.path().join("workspace"))
            .err()
            .unwrap();
        assert_eq!(resumed.code, ErrorCode::AlreadyOpen);
        let Fixture {
            temp,
            database,
            key,
        } = f;
        drop(database);
        let database = Database::open(&temp.path().join("workspace")).unwrap();
        let mut f = Fixture {
            temp,
            database,
            key,
        };
        assert_eq!(f.draft(chunk, None).unwrap(), first);
        assert!(f.summary().units.is_empty());
        let rejected = f
            .draft(
                json!({"kind":"append","draftId":group,"revision":2,
                "operationId":Uuid::new_v4(),"operations":[{"type":"organization","command":{
                    "type":"relate","id":Uuid::new_v4(),"kind":"prerequisite","from":unit,"to":unit
                }}]}),
                None,
            )
            .unwrap();
        assert_eq!(rejected["valid"], false);
        assert_eq!(rejected["operationIndex"], 3);
        let next = f.draft(json!({"kind":"append","draftId":group,"revision":2,
            "operationId":Uuid::new_v4(),"operations":[{"type":"organization","command":{
                "type":"createUnit","id":unit,"title":"Reading unit","objective":"Compare accounts","goalIds":[],"resourceIds":[resource]
            }}]}), None).unwrap();
        assert_eq!(next["revision"], 3);
        let sealed = f
            .draft(
                json!({"kind":"submit","draftId":group,"revision":3,"operationId":Uuid::new_v4()}),
                None,
            )
            .unwrap();
        assert_eq!(sealed["status"], "submitted");
        super::super::validate_native_data(&f.database.connection).unwrap();
        let review = f
            .review(json!({"kind":"inspect","draftId":group,"revision":4,"offset":0}))
            .unwrap();
        let mut apply = json!({"kind":"apply","draftId":group,"revision":4,"hash":"wrong","operationId":Uuid::new_v4()});
        assert_eq!(
            f.review(apply.clone()).unwrap_err().code,
            ErrorCode::RevisionConflict
        );
        assert!(f.summary().subjects.is_empty());
        apply["hash"] = review["hash"].clone();
        let applied = f.review(apply.clone()).unwrap();
        assert_eq!(applied["status"], "applied");
        assert_eq!(f.review(apply).unwrap(), applied);
        assert_eq!(f.summary().units.len(), 1);
        let resource = ResourceId::try_from(resource).unwrap();
        assert_eq!(
            f.database
                .read_document(RequestContext::local_ui(), resource, None)
                .unwrap()
                .unwrap()
                .authoring_origin,
            Some(crate::workspace::DocumentOrigin::ExternalUnverified)
        );
        super::super::validate_native_data(&f.database.connection).unwrap();

        assert_eq!(
            f.database
                .read_document(RequestContext::local_ui(), resource, None)
                .unwrap()
                .unwrap()
                .document
                .text(),
            "Synthetic text"
        );
    }

    #[test]
    fn mcp_course_delegation_spends_budget_once_and_failed_group_rolls_back() {
        let mut f = Fixture::new();
        let lease = Delegation::new(std::time::Duration::from_secs(30));
        let group = f.create("Course writing scope");
        let changes = (0..25)
            .map(|n| subject(Uuid::new_v4(), &format!("Subject {n}")))
            .collect();
        let validation = f.submit(group, changes);
        let first = json!({"kind":"apply_delegated","draftId":group,"revision":3,
            "hash":validation["hash"],"operationId":Uuid::new_v4()});
        let committed = f.draft(first.clone(), Some(&lease)).unwrap();
        assert_eq!(f.draft(first, Some(&lease)).unwrap(), committed);
        let group = f.create("Exceeds remaining budget");
        let changes = (0..8)
            .map(|n| subject(Uuid::new_v4(), &format!("Other subject {n}")))
            .collect();
        let validation = f.submit(group, changes);
        let second = json!({"kind":"apply_delegated","draftId":group,"revision":3,
            "hash":validation["hash"],"operationId":Uuid::new_v4()});
        assert_eq!(
            f.draft(second, Some(&lease)).unwrap_err().code,
            ErrorCode::BudgetExceeded
        );
        assert_eq!(f.summary().subjects.len(), 25);
        let current = f
            .draft(json!({"kind":"read","draftId":group,"offset":0}), None)
            .unwrap();
        assert_eq!(current["draft"]["status"], "submitted");
    }

    #[test]
    fn mcp_course_delegation_blocks_protected_writes_checks_revocation_and_undoes_additions() {
        let mut f = Fixture::new();
        let first = f.create("Delegated additions");
        let original = Uuid::new_v4();
        let validation = f.submit(first, vec![subject(original, "Language")]);
        let apply = json!({"kind":"apply_delegated","draftId":first,"revision":3,
            "hash":validation["hash"],"operationId":Uuid::new_v4()});
        assert_eq!(
            f.draft(apply.clone(), None).unwrap_err().code,
            ErrorCode::ApprovalRequired
        );
        let revoked = Delegation::new(std::time::Duration::from_secs(30));
        revoked.revoke();
        assert_eq!(
            f.draft(apply.clone(), Some(&revoked)).unwrap_err().code,
            ErrorCode::PermissionDenied
        );
        let valid = Delegation::new(std::time::Duration::from_secs(30));
        let applied = f.draft(apply.clone(), Some(&valid)).unwrap();
        assert_eq!(applied["status"], "applied");
        assert_eq!(f.draft(apply, Some(&valid)).unwrap(), applied);

        let protected = f.create("Protected update");
        let validation = f.submit(
            protected,
            vec![json!({"type":"organization","command":{
                "type":"updateSubject","id":original,"title":"Unexpected title"
            }})],
        );
        let denied = f
            .draft(
                json!({"kind":"apply_delegated","draftId":protected,
            "revision":3,"operationId":Uuid::new_v4(),"hash":validation["hash"]}),
                Some(&valid),
            )
            .unwrap_err();
        assert_eq!(denied.code, ErrorCode::ApprovalRequired);
        assert_eq!(
            f.summary().subjects.values().next().unwrap().title,
            "Language"
        );

        let undo = json!({"kind":"undo","draftId":first,"revision":4,"operationId":Uuid::new_v4()});
        let result = f.review(undo.clone()).unwrap();
        assert_eq!(result["status"], "undone");
        assert_eq!(f.review(undo).unwrap(), result);
        assert!(f.summary().subjects.values().next().unwrap().archived);
        assert_eq!(
            f.review(json!({"kind":"inspect","draftId":protected,"revision":3,"offset":0}))
                .unwrap()["validation"]["valid"],
            false
        );
    }
}
