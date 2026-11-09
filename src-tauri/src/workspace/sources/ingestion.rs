//! Durable, bounded staging of a selected immutable original.
use super::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateIngestionJob {
    pub contract_version: u32,
    pub workspace_id: WorkspaceId,
    pub operation_id: OperationId,
    pub source_id: Uuid,
    pub source_revision: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CancelIngestionJob {
    pub contract_version: u32,
    pub workspace_id: WorkspaceId,
    pub operation_id: OperationId,
    pub job_id: Uuid,
    pub expected_revision: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IngestionMapping {
    pub key_column: usize,
    pub title_column: usize,
    pub body_column: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ApplyIngestionJob {
    pub contract_version: u32,
    pub workspace_id: WorkspaceId,
    pub operation_id: OperationId,
    pub job_id: Uuid,
    pub expected_revision: u64,
    pub expected_library_revision: u64,
    pub mapping: IngestionMapping,
    pub selected: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ApplyReimportJob {
    pub contract_version: u32,
    pub workspace_id: WorkspaceId,
    pub operation_id: OperationId,
    pub job_id: Uuid,
    pub previous_job_id: Uuid,
    pub expected_revision: u64,
    pub expected_previous_revision: u64,
    pub expected_library_revision: u64,
    pub mapping: IngestionMapping,
    pub selected: Vec<usize>,
    pub resolutions: Vec<ReimportResolution>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReimportResolution {
    pub index: usize,
    pub expected_document_revision_id: Uuid,
    pub expected_title: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IngestionOutcome {
    pub index: usize,
    pub key: String,
    pub resource_id: ResourceId,
    pub document_revision_id: Uuid,
    pub before_revision_id: Option<Uuid>,
    pub before_title: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ReimportStatus {
    Unchanged,
    New,
    Changed,
    Conflict,
    Removed,
    Unresolved,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReimportItem {
    pub index: Option<usize>,
    pub key: String,
    pub resource_id: Option<ResourceId>,
    pub status: ReimportStatus,
    pub current_revision_id: Option<Uuid>,
    pub current_title: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReimportPreview {
    pub job_id: Uuid,
    pub previous_job_id: Uuid,
    pub items: Vec<ReimportItem>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IngestionStatus {
    Staged,
    Cancelled,
    Undone,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UndoIngestionJob {
    pub contract_version: u32,
    pub workspace_id: WorkspaceId,
    pub operation_id: OperationId,
    pub job_id: Uuid,
    pub expected_revision: u64,
    pub expected_library_revision: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UndoPreview {
    pub job_id: Uuid,
    pub can_undo: bool,
    pub resources: Vec<ResourceId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IngestionJob {
    pub id: Uuid,
    pub handler_version: u32,
    pub revision: u64,
    pub status: IngestionStatus,
    pub previous_job_id: Option<Uuid>,
    pub source: OriginalSource,
    pub extraction: SourceExtraction,
    pub mapping: Option<IngestionMapping>,
    pub outcomes: Vec<IngestionOutcome>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IngestionJobSummary {
    pub id: Uuid,
    pub source_id: Uuid,
    pub revision: u64,
    pub status: IngestionStatus,
    pub record_count: u64,
    pub applied_count: u64,
}

impl Database {
    pub(in crate::workspace) fn list_ingestion_jobs(
        &self,
        context: RequestContext,
        resource_id: ResourceId,
        offset: u64,
    ) -> Result<Vec<IngestionJobSummary>> {
        context.authorize()?;
        storage::resource(&self.connection, resource_id, false)?;
        if offset > super::super::types::MAX_REVISION as u64 {
            return Err(invalid());
        }
        let mut query = self.connection.prepare("SELECT json_object('id',j.id,'sourceId',j.source_id,'revision',json_extract(j.data,'$.revision'),'status',json_extract(j.data,'$.status'),'recordCount',json_array_length(json_extract(j.data,'$.extraction.records')),'appliedCount',json_array_length(json_extract(j.data,'$.outcomes'))) FROM ingestion_jobs j JOIN original_sources s ON s.id=j.source_id WHERE s.resource_id=?1 ORDER BY j.rowid LIMIT 20 OFFSET ?2")?;
        let rows = query.query_map(params![resource_id.to_string(), offset as i64], |r| {
            r.get::<_, String>(0)
        })?;
        rows.map(|row| serde_json::from_str(&row?).map_err(|_| corrupt()))
            .collect()
    }
    pub(in crate::workspace) fn preview_ingestion_undo(
        &self,
        context: RequestContext,
        job_id: Uuid,
    ) -> Result<UndoPreview> {
        let job = self.ingestion_job(context, job_id)?;
        self.verify_ingestion_job(&job)?;
        let mut can_undo = job.status != IngestionStatus::Undone && !job.outcomes.is_empty();
        let previous = job
            .previous_job_id
            .map(|id| self.ingestion_job(context, id))
            .transpose()?;
        if let Some(previous) = &previous {
            self.verify_ingestion_job(previous)?;
        }
        let descendant: bool = self.connection.query_row("SELECT EXISTS(SELECT 1 FROM ingestion_jobs WHERE json_extract(data,'$.previousJobId')=?1 AND json_extract(data,'$.status')<>'undone' AND json_array_length(json_extract(data,'$.outcomes'))>0)", [job.id.to_string()], |r| r.get(0))?;
        can_undo &= !descendant;
        let snapshot = self.library(context)?;
        for outcome in &job.outcomes {
            let old = previous
                .as_ref()
                .and_then(|p| p.outcomes.iter().find(|x| x.key == outcome.key));
            if old.is_some_and(|x| x.resource_id != outcome.resource_id) {
                return Err(corrupt());
            }
            if outcome.before_revision_id.is_some() != outcome.before_title.is_some() {
                return Err(corrupt());
            }
            if old.is_some() && outcome.before_revision_id.is_none() {
                if old.map(|x| x.document_revision_id) != Some(outcome.document_revision_id) {
                    return Err(corrupt());
                }
                continue;
            }
            let fields = &job
                .extraction
                .records
                .get(outcome.index)
                .ok_or_else(corrupt)?
                .fields;
            let (_, title, _) = mapped_fields(fields, job.mapping.as_ref().ok_or_else(corrupt)?)
                .ok_or_else(corrupt)?;
            let resource = snapshot
                .resources
                .get(&outcome.resource_id)
                .ok_or_else(corrupt)?;
            let head = self
                .read_document(context, outcome.resource_id, None)?
                .ok_or_else(corrupt)?;
            let dependent: bool = self.connection.query_row("SELECT EXISTS(SELECT 1 FROM annotations WHERE resource_id=?1 UNION ALL SELECT 1 FROM reading_positions WHERE resource_id=?1 UNION ALL SELECT 1 FROM unit_resources WHERE resource_id=?1 UNION ALL SELECT 1 FROM placements WHERE resource_id=?1 UNION ALL SELECT 1 FROM tag_resources WHERE resource_id=?1 UNION ALL SELECT 1 FROM original_sources WHERE resource_id=?1 UNION ALL SELECT 1 FROM lesson_resource_owners WHERE resource_id=?1 UNION ALL SELECT 1 FROM lesson_dependencies WHERE kind='resource' AND target_id=?1)", [outcome.resource_id.to_string()], |r| r.get(0))?;
            can_undo &= !resource.archived
                && resource.title == title
                && head.id == outcome.document_revision_id
                && !dependent;
        }
        Ok(UndoPreview {
            job_id,
            can_undo,
            resources: job.outcomes.iter().map(|x| x.resource_id).collect(),
        })
    }

    pub(in crate::workspace) fn undo_ingestion_job(
        &mut self,
        context: RequestContext,
        request: &UndoIngestionJob,
    ) -> Result<IngestionJob> {
        self.authorize_workspace(context, &request.workspace_id)?;
        if request.contract_version != 1 {
            return Err(invalid());
        }
        let hash = operations::request_hash("ingestion-undo", request)?;
        if let Some(receipt) =
            operations::exact_receipt(&self.connection, &self.id, &request.operation_id, &hash)?
        {
            return match receipt.result {
                ReceiptResult::Applied(OperationOutcome::IngestionJob(value)) => Ok(*value),
                ReceiptResult::Interrupted(code) => Err(AppError::new(code)),
                _ => Err(corrupt()),
            };
        }
        let mut job = self.ingestion_job(context, request.job_id)?;
        if job.revision != request.expected_revision
            || !self.preview_ingestion_undo(context, job.id)?.can_undo
            || self.library(context)?.revision != request.expected_library_revision
        {
            return Err(AppError::new(ErrorCode::RevisionConflict));
        }
        let previous = job
            .previous_job_id
            .map(|id| self.ingestion_job(context, id))
            .transpose()?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut revision = request.expected_library_revision;
        for outcome in &job.outcomes {
            let old = previous
                .as_ref()
                .and_then(|p| p.outcomes.iter().find(|x| x.key == outcome.key));
            if old.is_some() && outcome.before_revision_id.is_none() {
                continue;
            }
            let command = if old.is_some() {
                let title = outcome.before_title.as_ref().ok_or_else(corrupt)?;
                crate::organization::Command::RenameResource {
                    id: outcome.resource_id,
                    title: title.clone(),
                }
            } else {
                crate::organization::Command::Archive {
                    target: crate::organization::ArchiveTarget::Resource(outcome.resource_id),
                    archived: true,
                }
            };
            revision = super::super::content::apply_command(
                &tx,
                &self.id,
                &ApplyContent {
                    contract_version: 1,
                    workspace_id: self.id.clone(),
                    operation_id: request.operation_id.clone(),
                    expected_revision: revision,
                    command,
                },
            )?
            .revision;
            if old.is_some() {
                let current =
                    super::super::content::documents::read(&tx, outcome.resource_id, None)?
                        .ok_or_else(corrupt)?;
                let prior = super::super::content::documents::read(
                    &tx,
                    outcome.resource_id,
                    Some(outcome.before_revision_id.ok_or_else(corrupt)?),
                )?
                .ok_or_else(corrupt)?;
                super::super::content::documents::save_revision(
                    &tx,
                    &SaveDocument {
                        contract_version: 1,
                        workspace_id: self.id.clone(),
                        operation_id: request.operation_id.clone(),
                        resource_id: outcome.resource_id,
                        expected_revision: current.revision,
                        document: prior.document,
                    },
                )?;
            }
        }
        job.status = IngestionStatus::Undone;
        job.revision = job.revision.checked_add(1).ok_or_else(invalid)?;
        let data = serde_json::to_string(&job).map_err(|_| invalid())?;
        tx.execute(
            "UPDATE ingestion_jobs SET data=?1 WHERE id=?2",
            params![data, job.id.to_string()],
        )?;
        let receipt = operations::applied(
            request.operation_id.clone(),
            OperationOutcome::IngestionJob(Box::new(job.clone())),
        );
        operations::insert_applied(&tx, &self.id, &receipt, &hash, "ingestion")?;
        operations::durable_commit(tx)?;
        Ok(job)
    }
    pub(in crate::workspace) fn apply_ingestion_reimport(
        &mut self,
        context: RequestContext,
        request: &ApplyReimportJob,
    ) -> Result<IngestionJob> {
        self.authorize_workspace(context, &request.workspace_id)?;
        if request.contract_version != 1
            || request.selected.is_empty()
            || request.selected.len() > 100
        {
            return Err(invalid());
        }
        let hash = operations::request_hash("ingestion-reimport", request)?;
        if let Some(receipt) =
            operations::exact_receipt(&self.connection, &self.id, &request.operation_id, &hash)?
        {
            return match receipt.result {
                ReceiptResult::Applied(OperationOutcome::IngestionJob(value)) => Ok(*value),
                ReceiptResult::Interrupted(code) => Err(AppError::new(code)),
                _ => Err(corrupt()),
            };
        }
        let mut job = self.ingestion_job(context, request.job_id)?;
        let previous = self.ingestion_job(context, request.previous_job_id)?;
        if job.revision != request.expected_revision
            || previous.revision != request.expected_previous_revision
            || job.status != IngestionStatus::Staged
            || (job.previous_job_id.is_none() && !job.outcomes.is_empty())
            || job.previous_job_id.is_some_and(|id| id != previous.id)
            || job.mapping.as_ref().is_some_and(|m| m != &request.mapping)
        {
            return Err(AppError::new(ErrorCode::RevisionConflict));
        }
        let snapshot = self.library(context)?;
        if snapshot.revision != request.expected_library_revision {
            return Err(AppError::new(ErrorCode::RevisionConflict));
        }
        let preview =
            self.preview_ingestion_reimport(context, job.id, previous.id, &request.mapping)?;
        let resolutions: std::collections::HashMap<_, _> =
            request.resolutions.iter().map(|x| (x.index, x)).collect();
        if resolutions.len() != request.resolutions.len()
            || resolutions.len() > request.selected.len()
        {
            return Err(invalid());
        }
        let mut seen = std::collections::HashSet::new();
        let mut pending = Vec::new();
        for &index in &request.selected {
            if !seen.insert(index) || job.outcomes.iter().any(|x| x.index == index) {
                return Err(invalid());
            }
            let item = preview
                .items
                .iter()
                .find(|x| x.index == Some(index))
                .ok_or_else(invalid)?;
            if item.status == ReimportStatus::Conflict {
                let resolution = resolutions
                    .get(&index)
                    .ok_or_else(|| AppError::new(ErrorCode::RevisionConflict))?;
                if item.current_revision_id != Some(resolution.expected_document_revision_id)
                    || item.current_title.as_deref() != Some(resolution.expected_title.as_str())
                {
                    return Err(AppError::new(ErrorCode::RevisionConflict));
                }
            } else if !matches!(
                item.status,
                ReimportStatus::New | ReimportStatus::Changed | ReimportStatus::Unchanged
            ) || resolutions.contains_key(&index)
            {
                return Err(AppError::new(ErrorCode::RevisionConflict));
            }
            let fields = &job
                .extraction
                .records
                .get(index)
                .ok_or_else(invalid)?
                .fields;
            let (key, title, body) = mapped_fields(fields, &request.mapping).ok_or_else(invalid)?;
            let value = serde_json::json!({"schemaVersion":1,"blocks":[{"id":Uuid::new_v4(),"content":{"type":"paragraph","inlines":[{"text":body,"marks":[]}]}}]});
            let document = crate::document::Document::parse(
                &serde_json::to_vec(&value).map_err(|_| invalid())?,
            )
            .map_err(|_| invalid())?;
            pending.push((
                index,
                key.to_string(),
                title.to_string(),
                document,
                item.clone(),
            ));
        }
        if resolutions.keys().any(|index| !seen.contains(index)) {
            return Err(invalid());
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let fork: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM ingestion_jobs WHERE id<>?1 AND json_extract(data,'$.previousJobId')=?2 AND json_extract(data,'$.status')<>'undone' AND json_array_length(json_extract(data,'$.outcomes'))>0)", params![job.id.to_string(), previous.id.to_string()], |r| r.get(0))?;
        if fork {
            return Err(AppError::new(ErrorCode::RevisionConflict));
        }
        let mut revision = snapshot.revision;
        for (index, key, title, document, item) in pending {
            let (before_revision_id, before_title) = if matches!(
                item.status,
                ReimportStatus::Changed | ReimportStatus::Conflict
            ) {
                let id = item.resource_id.ok_or_else(corrupt)?;
                let head =
                    super::super::content::documents::read(&tx, id, None)?.ok_or_else(corrupt)?;
                let current = snapshot.resources.get(&id).ok_or_else(corrupt)?;
                (Some(head.id), Some(current.title.clone()))
            } else {
                (None, None)
            };
            let (resource_id, document_revision_id) = if item.status == ReimportStatus::Unchanged {
                let old = previous
                    .outcomes
                    .iter()
                    .find(|x| x.key == key)
                    .ok_or_else(corrupt)?;
                (old.resource_id, old.document_revision_id)
            } else {
                let resource_id = item.resource_id.unwrap_or_else(ResourceId::generate);
                let command = if item.status == ReimportStatus::New {
                    crate::organization::Command::CreateResource {
                        id: resource_id,
                        title,
                    }
                } else {
                    crate::organization::Command::RenameResource {
                        id: resource_id,
                        title,
                    }
                };
                revision = super::super::content::apply_command(
                    &tx,
                    &self.id,
                    &ApplyContent {
                        contract_version: 1,
                        workspace_id: self.id.clone(),
                        operation_id: request.operation_id.clone(),
                        expected_revision: revision,
                        command,
                    },
                )?
                .revision;
                let expected_revision = if item.status == ReimportStatus::New {
                    0
                } else {
                    super::super::content::documents::read(&tx, resource_id, None)?
                        .ok_or_else(corrupt)?
                        .revision
                };
                let saved = super::super::content::documents::save_revision(
                    &tx,
                    &SaveDocument {
                        contract_version: 1,
                        workspace_id: self.id.clone(),
                        operation_id: request.operation_id.clone(),
                        resource_id,
                        expected_revision,
                        document,
                    },
                )?;
                (resource_id, saved.id)
            };
            job.outcomes.push(IngestionOutcome {
                index,
                key,
                resource_id,
                document_revision_id,
                before_revision_id,
                before_title,
            });
        }
        job.previous_job_id = Some(previous.id);
        job.mapping = Some(request.mapping.clone());
        job.revision = job.revision.checked_add(1).ok_or_else(invalid)?;
        let data = serde_json::to_string(&job).map_err(|_| invalid())?;
        tx.execute(
            "UPDATE ingestion_jobs SET data=?1 WHERE id=?2",
            params![data, job.id.to_string()],
        )?;
        let receipt = operations::applied(
            request.operation_id.clone(),
            OperationOutcome::IngestionJob(Box::new(job.clone())),
        );
        operations::insert_applied(&tx, &self.id, &receipt, &hash, "ingestion")?;
        operations::durable_commit(tx)?;
        Ok(job)
    }
    pub(in crate::workspace) fn preview_ingestion_reimport(
        &self,
        context: RequestContext,
        job_id: Uuid,
        previous_job_id: Uuid,
        mapping: &IngestionMapping,
    ) -> Result<ReimportPreview> {
        let job = self.ingestion_job(context, job_id)?;
        let previous = self.ingestion_job(context, previous_job_id)?;
        self.verify_ingestion_job(&job)?;
        self.verify_ingestion_job(&previous)?;
        if job.id == previous.id
            || job.source.resource_id != previous.source.resource_id
            || previous.mapping.as_ref() != Some(mapping)
            || job.extraction.status != ExtractionStatus::Complete
            || previous.outcomes.is_empty()
        {
            return Err(invalid());
        }
        let snapshot = self.library(context)?;
        let mut counts = std::collections::HashMap::new();
        for record in &job.extraction.records {
            if let Some(key) = record.fields.get(mapping.key_column) {
                *counts.entry(key.trim().to_string()).or_insert(0usize) += 1;
            }
        }
        let mut seen = std::collections::HashSet::new();
        let old_by_key: std::collections::HashMap<_, _> = previous
            .outcomes
            .iter()
            .map(|outcome| (outcome.key.as_str(), outcome))
            .collect();
        if old_by_key.len() != previous.outcomes.len() {
            return Err(corrupt());
        }
        let mut items = Vec::new();
        for (index, record) in job.extraction.records.iter().enumerate() {
            let Some((key, title, body)) = mapped_fields(&record.fields, mapping) else {
                items.push(ReimportItem {
                    index: Some(index),
                    key: String::new(),
                    resource_id: None,
                    status: ReimportStatus::Unresolved,
                    current_revision_id: None,
                    current_title: None,
                });
                continue;
            };
            if key.is_empty() || counts.get(key).copied().unwrap_or(0) != 1 {
                if !key.is_empty() {
                    seen.insert(key.to_string());
                }
                items.push(ReimportItem {
                    index: Some(index),
                    key: key.into(),
                    resource_id: None,
                    status: ReimportStatus::Unresolved,
                    current_revision_id: None,
                    current_title: None,
                });
                continue;
            }
            seen.insert(key.to_string());
            let mut current_revision_id = None;
            let mut current_title = None;
            let old = old_by_key.get(key).copied();
            let (resource_id, status) = if let Some(old) = old {
                let fields = &previous
                    .extraction
                    .records
                    .get(old.index)
                    .ok_or_else(corrupt)?
                    .fields;
                let (_, old_title, old_body) =
                    mapped_fields(fields, mapping).ok_or_else(corrupt)?;
                if title == old_title && body == old_body {
                    (Some(old.resource_id), ReimportStatus::Unchanged)
                } else {
                    let resource = snapshot
                        .resources
                        .get(&old.resource_id)
                        .ok_or_else(corrupt)?;
                    let head = self
                        .read_document(context, old.resource_id, None)?
                        .ok_or_else(corrupt)?;
                    let edited = resource.archived
                        || resource.title != old_title
                        || head.id != old.document_revision_id;
                    if edited {
                        current_revision_id = Some(head.id);
                        current_title = Some(resource.title.clone());
                    }
                    (
                        Some(old.resource_id),
                        if edited {
                            ReimportStatus::Conflict
                        } else {
                            ReimportStatus::Changed
                        },
                    )
                }
            } else {
                (None, ReimportStatus::New)
            };
            items.push(ReimportItem {
                index: Some(index),
                key: key.into(),
                resource_id,
                status,
                current_revision_id,
                current_title,
            });
        }
        for old in &previous.outcomes {
            if !seen.contains(&old.key) {
                items.push(ReimportItem {
                    index: None,
                    key: old.key.clone(),
                    resource_id: Some(old.resource_id),
                    status: ReimportStatus::Removed,
                    current_revision_id: None,
                    current_title: None,
                });
            }
        }
        Ok(ReimportPreview {
            job_id,
            previous_job_id,
            items,
        })
    }
    pub(in crate::workspace) fn create_ingestion_job(
        &mut self,
        context: RequestContext,
        request: &CreateIngestionJob,
    ) -> Result<IngestionJob> {
        self.authorize_workspace(context, &request.workspace_id)?;
        if request.contract_version != 1 || request.source_id.is_nil() {
            return Err(invalid());
        }
        let hash = operations::request_hash("ingestion-create", request)?;
        if let Some(receipt) =
            operations::exact_receipt(&self.connection, &self.id, &request.operation_id, &hash)?
        {
            return match receipt.result {
                ReceiptResult::Applied(OperationOutcome::IngestionJob(value)) => Ok(*value),
                ReceiptResult::Interrupted(code) => Err(AppError::new(code)),
                _ => Err(corrupt()),
            };
        }
        let source = storage::find(&self.connection, request.source_id)?;
        if source.revision != request.source_revision {
            return Err(AppError::new(ErrorCode::RevisionConflict));
        }
        let info = AssetInfo {
            digest: source.digest.clone(),
            byte_length: source.byte_length,
        };
        assets::verify(self.owner.root(), &info).map_err(|_| corrupt())?;
        let bytes = preview::read_bytes(self.owner.root(), &info, 8 * 1024 * 1024 + 1)?;
        let extraction = extraction::extract_for_ingestion(&source.filename, &bytes);
        if source.byte_length == bytes.len() as u64
            && migrations::checksum(&bytes) != source.digest.as_str()
        {
            return Err(corrupt());
        }
        let job = IngestionJob {
            id: Uuid::new_v4(),
            handler_version: 1,
            revision: 0,
            status: IngestionStatus::Staged,
            previous_job_id: None,
            source,
            extraction,
            mapping: None,
            outcomes: Vec::new(),
        };
        let data = serde_json::to_string(&job).map_err(|_| invalid())?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute("INSERT INTO ingestion_jobs(id,operation_id,request_hash,source_id,data) VALUES(?1,?2,?3,?4,?5)",
            params![job.id.to_string(), request.operation_id.as_str(), hash, job.source.id.to_string(), data])?;
        let receipt = operations::applied(
            request.operation_id.clone(),
            OperationOutcome::IngestionJob(Box::new(job.clone())),
        );
        operations::insert_applied(&tx, &self.id, &receipt, &hash, "ingestion")?;
        operations::durable_commit(tx)?;
        Ok(job)
    }

    pub(in crate::workspace) fn ingestion_job(
        &self,
        context: RequestContext,
        id: Uuid,
    ) -> Result<IngestionJob> {
        context.authorize()?;
        if id.is_nil() {
            return Err(invalid());
        }
        let data: String = self
            .connection
            .query_row(
                "SELECT data FROM ingestion_jobs WHERE id=?1",
                [id.to_string()],
                |r| r.get(0),
            )
            .optional()?
            .ok_or_else(invalid)?;
        let job: IngestionJob = serde_json::from_str(&data).map_err(|_| corrupt())?;
        if job.id != id {
            return Err(corrupt());
        }
        Ok(job)
    }

    pub(in crate::workspace) fn cancel_ingestion_job(
        &mut self,
        context: RequestContext,
        request: &CancelIngestionJob,
    ) -> Result<IngestionJob> {
        self.authorize_workspace(context, &request.workspace_id)?;
        if request.contract_version != 1 || request.job_id.is_nil() {
            return Err(invalid());
        }
        let hash = operations::request_hash("ingestion-cancel", request)?;
        if let Some(receipt) =
            operations::exact_receipt(&self.connection, &self.id, &request.operation_id, &hash)?
        {
            return match receipt.result {
                ReceiptResult::Applied(OperationOutcome::IngestionJob(value)) => Ok(*value),
                ReceiptResult::Interrupted(code) => Err(AppError::new(code)),
                _ => Err(corrupt()),
            };
        }
        let mut job = self.ingestion_job(context, request.job_id)?;
        if job.revision != request.expected_revision || job.status != IngestionStatus::Staged {
            return Err(AppError::new(ErrorCode::RevisionConflict));
        }
        job.revision = job.revision.checked_add(1).ok_or_else(invalid)?;
        job.status = IngestionStatus::Cancelled;
        let data = serde_json::to_string(&job).map_err(|_| invalid())?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute(
            "UPDATE ingestion_jobs SET data=?1 WHERE id=?2",
            params![data, job.id.to_string()],
        )?;
        let receipt = operations::applied(
            request.operation_id.clone(),
            OperationOutcome::IngestionJob(Box::new(job.clone())),
        );
        operations::insert_applied(&tx, &self.id, &receipt, &hash, "ingestion")?;
        operations::durable_commit(tx)?;
        Ok(job)
    }

    pub(in crate::workspace) fn apply_ingestion_job(
        &mut self,
        context: RequestContext,
        request: &ApplyIngestionJob,
    ) -> Result<IngestionJob> {
        self.authorize_workspace(context, &request.workspace_id)?;
        if request.contract_version != 1
            || request.job_id.is_nil()
            || request.selected.is_empty()
            || request.selected.len() > 100
        {
            return Err(invalid());
        }
        let hash = operations::request_hash("ingestion-apply", request)?;
        if let Some(receipt) =
            operations::exact_receipt(&self.connection, &self.id, &request.operation_id, &hash)?
        {
            return match receipt.result {
                ReceiptResult::Applied(OperationOutcome::IngestionJob(value)) => Ok(*value),
                ReceiptResult::Interrupted(code) => Err(AppError::new(code)),
                _ => Err(corrupt()),
            };
        }
        let mut job = self.ingestion_job(context, request.job_id)?;
        self.verify_ingestion_job(&job)?;
        if job.revision != request.expected_revision || job.status != IngestionStatus::Staged {
            return Err(AppError::new(ErrorCode::RevisionConflict));
        }
        if job.extraction.status != ExtractionStatus::Complete
            || job.mapping.as_ref().is_some_and(|m| m != &request.mapping)
        {
            return Err(invalid());
        }
        let source = job.source.clone();
        let mut indices = std::collections::HashSet::new();
        let mut keys: std::collections::HashSet<String> =
            job.outcomes.iter().map(|x| x.key.clone()).collect();
        let mut pending = Vec::new();
        for &index in &request.selected {
            if !indices.insert(index) || job.outcomes.iter().any(|outcome| outcome.index == index) {
                return Err(invalid());
            }
            let fields = &job
                .extraction
                .records
                .get(index)
                .ok_or_else(invalid)?
                .fields;
            let key = fields
                .get(request.mapping.key_column)
                .ok_or_else(invalid)?
                .trim()
                .to_string();
            let title = fields
                .get(request.mapping.title_column)
                .ok_or_else(invalid)?
                .trim()
                .to_string();
            let body = fields
                .get(request.mapping.body_column)
                .ok_or_else(invalid)?
                .clone();
            if key.is_empty() || key.len() > 1024 || !keys.insert(key.clone()) {
                return Err(invalid());
            }
            let value = serde_json::json!({"schemaVersion":1,"blocks":[{"id":Uuid::new_v4(),"content":{"type":"paragraph","inlines":[{"text":body,"marks":[]}]}}]});
            let document = crate::document::Document::parse(
                &serde_json::to_vec(&value).map_err(|_| invalid())?,
            )
            .map_err(|_| invalid())?;
            pending.push((index, key, title, document));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let duplicate: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM ingestion_jobs j JOIN original_sources s ON s.id=j.source_id WHERE j.id<>?1 AND s.resource_id=?2 AND json_array_length(json_extract(j.data,'$.outcomes'))>0)",
            params![job.id.to_string(), source.resource_id.to_string()],
            |r| r.get(0),
        )?;
        if duplicate {
            return Err(invalid());
        }
        let mut revision = request.expected_library_revision;
        for (index, key, title, document) in pending {
            let resource_id = ResourceId::generate();
            let command = ApplyContent {
                contract_version: 1,
                workspace_id: self.id.clone(),
                operation_id: request.operation_id.clone(),
                expected_revision: revision,
                command: crate::organization::Command::CreateResource {
                    id: resource_id,
                    title,
                },
            };
            let snapshot = super::super::content::apply_command(&tx, &self.id, &command)?;
            revision = snapshot.revision;
            let saved = super::super::content::documents::save_revision(
                &tx,
                &SaveDocument {
                    contract_version: 1,
                    workspace_id: self.id.clone(),
                    operation_id: request.operation_id.clone(),
                    resource_id,
                    expected_revision: 0,
                    document,
                },
            )?;
            job.outcomes.push(IngestionOutcome {
                index,
                key,
                resource_id,
                document_revision_id: saved.id,
                before_revision_id: None,
                before_title: None,
            });
        }
        job.revision = job.revision.checked_add(1).ok_or_else(invalid)?;
        job.mapping = Some(request.mapping.clone());
        let data = serde_json::to_string(&job).map_err(|_| invalid())?;
        tx.execute(
            "UPDATE ingestion_jobs SET data=?1 WHERE id=?2",
            params![data, job.id.to_string()],
        )?;
        let receipt = operations::applied(
            request.operation_id.clone(),
            OperationOutcome::IngestionJob(Box::new(job.clone())),
        );
        operations::insert_applied(&tx, &self.id, &receipt, &hash, "ingestion")?;
        operations::durable_commit(tx)?;
        Ok(job)
    }

    pub(in crate::workspace) fn verify_ingestion_job(&self, job: &IngestionJob) -> Result<()> {
        let source = storage::find(&self.connection, job.source.id)?;
        if source != job.source || job.handler_version != 1 {
            return Err(corrupt());
        }
        let info = AssetInfo {
            digest: source.digest.clone(),
            byte_length: source.byte_length,
        };
        assets::verify(self.owner.root(), &info).map_err(|_| corrupt())?;
        let bytes = preview::read_bytes(self.owner.root(), &info, 8 * 1024 * 1024 + 1)?;
        if (source.byte_length == bytes.len() as u64
            && migrations::checksum(&bytes) != source.digest.as_str())
            || extraction::extract_for_ingestion(&source.filename, &bytes) != job.extraction
        {
            return Err(corrupt());
        }
        Ok(())
    }
}

fn mapped_fields<'a>(
    fields: &'a [String],
    mapping: &IngestionMapping,
) -> Option<(&'a str, &'a str, &'a str)> {
    Some((
        fields.get(mapping.key_column)?.trim(),
        fields.get(mapping.title_column)?.trim(),
        fields.get(mapping.body_column)?,
    ))
}
