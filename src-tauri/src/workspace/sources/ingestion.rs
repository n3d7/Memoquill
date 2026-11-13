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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub json_properties: Option<JsonPropertyMapping>,
}

/// Required name-or-null selectors: an absent selector is not an implicit raw selection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct JsonPropertyMapping {
    #[serde(deserialize_with = "required_json_property")]
    pub key: Option<String>,
    #[serde(deserialize_with = "required_json_property")]
    pub title: Option<String>,
    #[serde(deserialize_with = "required_json_property")]
    pub body: Option<String>,
}
fn required_json_property<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<Option<String>, D::Error> {
    Option::deserialize(deserializer)
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
pub enum IngestionDisposition {
    Pending,
    Omitted,
    Empty,
    Unresolved,
    RemovedRetained,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IngestionDecision {
    pub index: Option<usize>,
    pub key: Option<String>,
    pub disposition: IngestionDisposition,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReviewIngestionJob {
    pub contract_version: u32,
    pub workspace_id: WorkspaceId,
    pub operation_id: OperationId,
    pub job_id: Uuid,
    pub expected_revision: u64,
    pub previous_job_id: Option<Uuid>,
    pub expected_previous_revision: Option<u64>,
    pub mapping: IngestionMapping,
    pub decisions: Vec<IngestionDecision>,
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
pub struct ReimportValues {
    pub title: String,
    pub body: String,
    pub locator: ExtractionLocator,
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
    pub current_body: Option<String>,
    pub prior: Option<ReimportValues>,
    pub incoming: Option<ReimportValues>,
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
    pub diagnostics: Vec<String>,
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
    #[serde(default)]
    pub decisions: Vec<IngestionDecision>,
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
    #[serde(default)]
    pub eligible_baseline: bool,
}

struct LineageItem {
    outcome: IngestionOutcome,
    locator: ExtractionLocator,
    title: String,
    body: String,
}

impl Database {
    pub(in crate::workspace) fn review_ingestion_job(
        &mut self,
        context: RequestContext,
        request: &ReviewIngestionJob,
    ) -> Result<IngestionJob> {
        self.authorize_workspace(context, &request.workspace_id)?;
        if request.contract_version != 1
            || request.decisions.is_empty()
            || request.decisions.len() > 100
            || request.previous_job_id.is_some() != request.expected_previous_revision.is_some()
        {
            return Err(invalid());
        }
        let hash = operations::request_hash("ingestion-review", request)?;
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
        if job.revision != request.expected_revision
            || job.status != IngestionStatus::Staged
            || job
                .mapping
                .as_ref()
                .is_some_and(|mapping| mapping != &request.mapping)
            || job
                .previous_job_id
                .is_some_and(|id| Some(id) != request.previous_job_id)
            || (request.previous_job_id.is_some()
                && job.previous_job_id.is_none()
                && !job.outcomes.is_empty())
        {
            return Err(AppError::new(ErrorCode::RevisionConflict));
        }
        let preview = if let Some(id) = request.previous_job_id {
            let previous = self.ingestion_job(context, id)?;
            if Some(previous.revision) != request.expected_previous_revision {
                return Err(AppError::new(ErrorCode::RevisionConflict));
            }
            Some(self.preview_ingestion_reimport(context, job.id, id, &request.mapping)?)
        } else {
            None
        };
        if (request.mapping.json_properties.is_some() && job.handler_version != 2)
            || job.extraction.status != ExtractionStatus::Complete
        {
            return Err(invalid());
        }
        let mut seen = std::collections::HashSet::new();
        for decision in &request.decisions {
            if !seen.insert((decision.index, decision.key.clone())) {
                return Err(invalid());
            }
            match (
                decision.index,
                decision.key.as_deref(),
                decision.disposition,
            ) {
                (Some(index), None, disposition)
                    if disposition != IngestionDisposition::RemovedRetained =>
                {
                    let record = job.extraction.records.get(index).ok_or_else(invalid)?;
                    if job.outcomes.iter().any(|outcome| outcome.index == index) {
                        return Err(invalid());
                    }
                    if disposition == IngestionDisposition::Empty
                        && record.fields.iter().any(|field| !field.trim().is_empty())
                    {
                        return Err(invalid());
                    }
                    if disposition == IngestionDisposition::Unresolved {
                        let unresolved = preview
                            .as_ref()
                            .and_then(|preview| {
                                preview.items.iter().find(|item| item.index == Some(index))
                            })
                            .is_some_and(|item| item.status == ReimportStatus::Unresolved)
                            || mapped_fields(record, &request.mapping).is_none_or(
                                |(key, title, _)| {
                                    key.is_empty()
                                        || title.is_empty()
                                        || key.len() > 1024
                                        || job
                                            .extraction
                                            .records
                                            .iter()
                                            .filter(|other| {
                                                mapped_fields(other, &request.mapping).is_some_and(
                                                    |(other_key, _, _)| other_key == key,
                                                )
                                            })
                                            .take(2)
                                            .count()
                                            > 1
                                },
                            );
                        if !unresolved {
                            return Err(invalid());
                        }
                    }
                }
                (
                    None,
                    Some(key),
                    IngestionDisposition::RemovedRetained | IngestionDisposition::Pending,
                ) => {
                    if !preview.as_ref().is_some_and(|preview| {
                        preview.items.iter().any(|item| {
                            item.index.is_none()
                                && item.key == key
                                && item.status == ReimportStatus::Removed
                        })
                    }) {
                        return Err(invalid());
                    }
                }
                _ => return Err(invalid()),
            }
            job.decisions.retain(|existing| {
                existing.index != decision.index || existing.key != decision.key
            });
            if decision.disposition != IngestionDisposition::Pending {
                job.decisions.push(decision.clone());
            }
        }
        job.decisions
            .sort_by(|a, b| (a.index, &a.key).cmp(&(b.index, &b.key)));
        job.previous_job_id = request.previous_job_id;
        job.mapping = Some(request.mapping.clone());
        job.revision = job.revision.checked_add(1).ok_or_else(invalid)?;
        let data = serde_json::to_string(&job).map_err(|_| invalid())?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        ensure_no_applied_descendants(&tx, job.id)?;
        if let Some(previous) = request.previous_job_id {
            ensure_no_lineage_fork(&tx, job.id, previous)?;
        }
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
    fn ingestion_lineage(
        &self,
        context: RequestContext,
        previous: &IngestionJob,
        mapping: &IngestionMapping,
    ) -> Result<std::collections::BTreeMap<String, LineageItem>> {
        let mut current = previous.clone();
        let mut visited = std::collections::HashSet::new();
        let mut lineage = std::collections::BTreeMap::new();
        let mut retained_bytes = 0usize;
        loop {
            if !visited.insert(current.id)
                || visited.len() > 256
                || current.status == IngestionStatus::Undone
                || current.source.resource_id != previous.source.resource_id
                || current.mapping.as_ref() != Some(mapping)
            {
                return Err(corrupt());
            }
            self.verify_ingestion_job(&current)?;
            let mut own_keys = std::collections::HashSet::new();
            for outcome in &current.outcomes {
                if !own_keys.insert(&outcome.key) {
                    return Err(corrupt());
                }
                let record = current
                    .extraction
                    .records
                    .get(outcome.index)
                    .ok_or_else(corrupt)?;
                let (key, title, body) = mapped_fields(record, mapping).ok_or_else(corrupt)?;
                if key != outcome.key || key.is_empty() {
                    return Err(corrupt());
                }
                if !lineage.contains_key(key) {
                    retained_bytes = retained_bytes
                        .checked_add(key.len() + title.len() + body.len())
                        .ok_or_else(invalid)?;
                    if retained_bytes > 16 * 1024 * 1024 || lineage.len() >= 10_000 {
                        return Err(AppError::new(ErrorCode::BudgetExceeded));
                    }
                    lineage.insert(
                        key.to_owned(),
                        LineageItem {
                            outcome: outcome.clone(),
                            locator: record.locator.clone(),
                            title: title.to_owned(),
                            body: body.to_owned(),
                        },
                    );
                }
            }
            match current.previous_job_id {
                Some(id) => current = self.ingestion_job(context, id)?,
                None => break,
            }
        }
        Ok(lineage)
    }
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
        let mut summaries: Vec<IngestionJobSummary> = rows
            .map(|row| serde_json::from_str(&row?).map_err(|_| corrupt()))
            .collect::<Result<_>>()?;
        for summary in &mut summaries {
            if summary.status == IngestionStatus::Undone
                || (summary.status == IngestionStatus::Cancelled && summary.applied_count == 0)
            {
                continue;
            }
            match ensure_no_applied_descendants(&self.connection, summary.id) {
                Ok(()) => {}
                Err(error) if error.code == ErrorCode::RevisionConflict => continue,
                Err(error) => return Err(error),
            }
            let job = self.ingestion_job(context, summary.id)?;
            if let Some(mapping) = &job.mapping {
                summary.eligible_baseline = self
                    .ingestion_lineage(context, &job, mapping)
                    .is_ok_and(|lineage| !lineage.is_empty());
            }
        }
        Ok(summaries)
    }
    pub(in crate::workspace) fn preview_ingestion_undo(
        &self,
        context: RequestContext,
        job_id: Uuid,
    ) -> Result<UndoPreview> {
        let job = self.ingestion_job(context, job_id)?;
        self.verify_ingestion_job(&job)?;
        let mut can_undo = job.status != IngestionStatus::Undone && !job.outcomes.is_empty();
        let mut diagnostics = Vec::new();
        let mut resources = Vec::new();
        if !can_undo {
            diagnostics.push("There are no applied changes available to undo.".into());
        }
        let previous = job
            .previous_job_id
            .map(|id| self.ingestion_job(context, id))
            .transpose()?;
        let lineage = previous
            .as_ref()
            .map(|previous| {
                self.ingestion_lineage(context, previous, job.mapping.as_ref().ok_or_else(corrupt)?)
            })
            .transpose()?;
        let descendant: bool = self.connection.query_row("SELECT EXISTS(SELECT 1 FROM ingestion_jobs WHERE json_extract(data,'$.previousJobId')=?1 AND json_extract(data,'$.status')<>'undone' AND (json_extract(data,'$.status')<>'cancelled' OR json_array_length(json_extract(data,'$.outcomes'))>0) AND json_extract(data,'$.revision')>0)", [job.id.to_string()], |r| r.get(0))?;
        can_undo &= !descendant;
        if descendant {
            diagnostics
                .push("A later import depends on this job. Undo the later import first.".into());
        }
        let snapshot = self.library(context)?;
        for outcome in &job.outcomes {
            let old = lineage
                .as_ref()
                .and_then(|items| items.get(&outcome.key))
                .map(|item| &item.outcome);
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
            let record = job
                .extraction
                .records
                .get(outcome.index)
                .ok_or_else(corrupt)?;
            let (_, title, _) = mapped_fields(record, job.mapping.as_ref().ok_or_else(corrupt)?)
                .ok_or_else(corrupt)?;
            let resource = snapshot
                .resources
                .get(&outcome.resource_id)
                .ok_or_else(corrupt)?;
            let head = self
                .read_document(context, outcome.resource_id, None)?
                .ok_or_else(corrupt)?;
            let dependent: bool = self.connection.query_row("SELECT EXISTS(SELECT 1 FROM annotations WHERE resource_id=?1 UNION ALL SELECT 1 FROM reading_positions WHERE resource_id=?1 UNION ALL SELECT 1 FROM unit_resources WHERE resource_id=?1 UNION ALL SELECT 1 FROM placements WHERE resource_id=?1 UNION ALL SELECT 1 FROM tag_resources WHERE resource_id=?1 UNION ALL SELECT 1 FROM original_sources WHERE resource_id=?1 UNION ALL SELECT 1 FROM lesson_resource_owners WHERE resource_id=?1 UNION ALL SELECT 1 FROM lesson_dependencies WHERE kind='resource' AND target_id=?1)", [outcome.resource_id.to_string()], |r| r.get(0))?;
            let draft_dependent: bool = self.connection.query_row(
                "SELECT EXISTS(SELECT 1 FROM lesson_drafts d, json_each(d.data,'$.draft.composition.bindings') b WHERE json_extract(b.value,'$.targetKind')='resource' AND json_extract(b.value,'$.targetId')=?1 UNION ALL SELECT 1 FROM lesson_drafts d, json_each(d.data,'$.dependencies') p WHERE json_extract(p.value,'$.kind')='Resource' AND json_extract(p.value,'$.target')=?1)",
                [outcome.resource_id.to_string()], |r| r.get(0),
            )?;
            let unchanged = !resource.archived
                && resource.title == title
                && head.id == outcome.document_revision_id;
            can_undo &= unchanged && !dependent && !draft_dependent;
            resources.push(outcome.resource_id);
            if !unchanged {
                diagnostics.push(format!(
                    "Material {} was edited or archived after this import.",
                    outcome.resource_id
                ));
            }
            if dependent {
                diagnostics.push(format!("Material {} has saved annotations, reading history, organization or lesson references.", outcome.resource_id));
            }
            if draft_dependent {
                diagnostics.push(format!(
                    "Material {} is referenced by a saved lesson draft.",
                    outcome.resource_id
                ));
            }
        }
        Ok(UndoPreview {
            job_id,
            can_undo,
            resources,
            diagnostics,
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
        let lineage = previous
            .as_ref()
            .map(|previous| {
                self.ingestion_lineage(context, previous, job.mapping.as_ref().ok_or_else(corrupt)?)
            })
            .transpose()?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut revision = request.expected_library_revision;
        for outcome in &job.outcomes {
            let old = lineage
                .as_ref()
                .and_then(|items| items.get(&outcome.key))
                .map(|item| &item.outcome);
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
        let lineage = self.ingestion_lineage(context, &previous, &request.mapping)?;
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
            if !seen.insert(index)
                || job.outcomes.iter().any(|x| x.index == index)
                || job.decisions.iter().any(|x| x.index == Some(index))
            {
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
            let record = job.extraction.records.get(index).ok_or_else(invalid)?;
            let (key, title, body) = mapped_fields(record, &request.mapping).ok_or_else(invalid)?;
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
        ensure_no_applied_descendants(&tx, job.id)?;
        ensure_no_lineage_fork(&tx, job.id, previous.id)?;
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
                let old = &lineage.get(&key).ok_or_else(corrupt)?.outcome;
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
        if (mapping.json_properties.is_some() && job.handler_version != 2)
            || job.id == previous.id
            || previous.status == IngestionStatus::Undone
            || (previous.status == IngestionStatus::Cancelled && previous.outcomes.is_empty())
            || job.source.resource_id != previous.source.resource_id
            || previous.mapping.as_ref() != Some(mapping)
            || job.extraction.status != ExtractionStatus::Complete
        {
            return Err(invalid());
        }
        let snapshot = self.library(context)?;
        let mut counts = std::collections::HashMap::new();
        for record in &job.extraction.records {
            if let Some(key) = mapped_key(record, mapping) {
                *counts.entry(key.trim().to_string()).or_insert(0usize) += 1;
            }
        }
        let mut seen = std::collections::HashSet::new();
        let old_by_key = self.ingestion_lineage(context, &previous, mapping)?;
        if old_by_key.is_empty() {
            return Err(invalid());
        }
        let mut items = Vec::new();
        let mut preview_bytes = 0usize;
        for (index, record) in job.extraction.records.iter().enumerate() {
            let fields = mapped_fields(record, mapping);
            let incoming = fields.map(|(_, title, body)| ReimportValues {
                title: title.into(),
                body: body.into(),
                locator: record.locator.clone(),
            });
            let key = fields.map(|(key, _, _)| key).unwrap_or("");
            if key.is_empty() || counts.get(key).copied().unwrap_or(0) != 1 {
                if !key.is_empty() {
                    seen.insert(key.to_string());
                }
                let item = ReimportItem {
                    index: Some(index),
                    key: key.into(),
                    resource_id: None,
                    status: ReimportStatus::Unresolved,
                    current_revision_id: None,
                    current_title: None,
                    current_body: None,
                    prior: None,
                    incoming,
                };
                preview_bytes = preview_bytes
                    .checked_add(comparison_bytes(&item))
                    .ok_or_else(invalid)?;
                if preview_bytes > 32 * 1024 * 1024 {
                    return Err(AppError::new(ErrorCode::BudgetExceeded));
                }
                items.push(item);
                continue;
            }
            seen.insert(key.to_string());
            let old = old_by_key.get(key);
            let mut item = ReimportItem {
                index: Some(index),
                key: key.into(),
                resource_id: None,
                status: ReimportStatus::New,
                current_revision_id: None,
                current_title: None,
                current_body: None,
                prior: old.map(|old| ReimportValues {
                    title: old.title.clone(),
                    body: old.body.clone(),
                    locator: old.locator.clone(),
                }),
                incoming,
            };
            if let Some(old) = old {
                let resource = snapshot
                    .resources
                    .get(&old.outcome.resource_id)
                    .ok_or_else(corrupt)?;
                let head = self
                    .read_document(context, old.outcome.resource_id, None)?
                    .ok_or_else(corrupt)?;
                let edited =
                    resource.title != old.title || head.id != old.outcome.document_revision_id;
                let (_, title, body) = fields.ok_or_else(corrupt)?;
                item.resource_id = Some(old.outcome.resource_id);
                item.current_revision_id = Some(head.id);
                item.current_title = Some(resource.title.clone());
                item.current_body = Some(head.document.text());
                item.status = if resource.archived {
                    ReimportStatus::Unresolved
                } else if title == old.title && body == old.body {
                    ReimportStatus::Unchanged
                } else if edited {
                    ReimportStatus::Conflict
                } else {
                    ReimportStatus::Changed
                };
            }
            preview_bytes = preview_bytes
                .checked_add(comparison_bytes(&item))
                .ok_or_else(invalid)?;
            if preview_bytes > 32 * 1024 * 1024 {
                return Err(AppError::new(ErrorCode::BudgetExceeded));
            }
            items.push(item);
        }
        for (key, old) in old_by_key {
            if !seen.contains(&key) {
                let resource = snapshot
                    .resources
                    .get(&old.outcome.resource_id)
                    .ok_or_else(corrupt)?;
                let head = self
                    .read_document(context, old.outcome.resource_id, None)?
                    .ok_or_else(corrupt)?;
                let item = ReimportItem {
                    index: None,
                    key,
                    resource_id: Some(old.outcome.resource_id),
                    status: ReimportStatus::Removed,
                    current_revision_id: Some(head.id),
                    current_title: Some(resource.title.clone()),
                    current_body: Some(head.document.text()),
                    prior: Some(ReimportValues {
                        title: old.title,
                        body: old.body,
                        locator: old.locator,
                    }),
                    incoming: None,
                };
                preview_bytes = preview_bytes
                    .checked_add(comparison_bytes(&item))
                    .ok_or_else(invalid)?;
                if preview_bytes > 32 * 1024 * 1024 {
                    return Err(AppError::new(ErrorCode::BudgetExceeded));
                }
                items.push(item);
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
            handler_version: extraction.contract_version,
            revision: 0,
            status: IngestionStatus::Staged,
            previous_job_id: None,
            source,
            extraction,
            mapping: None,
            outcomes: Vec::new(),
            decisions: Vec::new(),
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
        ensure_no_applied_descendants(&tx, job.id)?;
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
        if (request.mapping.json_properties.is_some() && job.handler_version != 2)
            || job.extraction.status != ExtractionStatus::Complete
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
            if !indices.insert(index)
                || job.outcomes.iter().any(|outcome| outcome.index == index)
                || job
                    .decisions
                    .iter()
                    .any(|decision| decision.index == Some(index))
            {
                return Err(invalid());
            }
            let record = job.extraction.records.get(index).ok_or_else(invalid)?;
            let (key, title, body) = mapped_fields(record, &request.mapping).ok_or_else(invalid)?;
            let (key, title, body) = (key.to_owned(), title.to_owned(), body.to_owned());
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
        ensure_no_applied_descendants(&tx, job.id)?;
        let duplicate: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM ingestion_jobs j JOIN original_sources s ON s.id=j.source_id WHERE j.id<>?1 AND s.resource_id=?2 AND s.digest=?3 AND json_extract(j.data,'$.status')<>'undone' AND json_array_length(json_extract(j.data,'$.outcomes'))>0)",
            params![job.id.to_string(), source.resource_id.to_string(), source.digest.as_str()],
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
        if source != job.source || !matches!(job.handler_version, 1 | 2) {
            return Err(corrupt());
        }
        let info = AssetInfo {
            digest: source.digest.clone(),
            byte_length: source.byte_length,
        };
        assets::verify(self.owner.root(), &info).map_err(|_| corrupt())?;
        let bytes = preview::read_bytes(self.owner.root(), &info, 8 * 1024 * 1024 + 1)?;
        let extraction = match job.handler_version {
            1 => extraction::extract_for_ingestion_legacy(&source.filename, &bytes),
            2 if job.extraction.format == ExtractionFormat::Json => {
                extraction::extract_for_ingestion(&source.filename, &bytes)
            }
            _ => return Err(corrupt()),
        };
        if (source.byte_length == bytes.len() as u64
            && migrations::checksum(&bytes) != source.digest.as_str())
            || extraction != job.extraction
        {
            return Err(corrupt());
        }
        Ok(())
    }
}

fn mapped_value<'a>(
    record: &'a ExtractionRecord,
    column: usize,
    property: Option<&Option<String>>,
) -> Option<&'a str> {
    if !record.duplicate_properties.is_empty() {
        return None;
    }
    match property {
        Some(Some(name)) => record.properties.get(name).map(String::as_str),
        Some(None) => record.fields.first().map(String::as_str),
        None => record.fields.get(column).map(String::as_str),
    }
}
fn mapped_key<'a>(record: &'a ExtractionRecord, mapping: &IngestionMapping) -> Option<&'a str> {
    mapped_value(
        record,
        mapping.key_column,
        mapping.json_properties.as_ref().map(|p| &p.key),
    )
    .map(str::trim)
}
fn mapped_fields<'a>(
    record: &'a ExtractionRecord,
    mapping: &IngestionMapping,
) -> Option<(&'a str, &'a str, &'a str)> {
    let properties = mapping.json_properties.as_ref();
    Some((
        mapped_key(record, mapping)?,
        mapped_value(record, mapping.title_column, properties.map(|p| &p.title))?.trim(),
        mapped_value(record, mapping.body_column, properties.map(|p| &p.body))?,
    ))
}

fn ensure_no_applied_descendants(connection: &Connection, id: Uuid) -> Result<()> {
    let exists: bool = connection.query_row("SELECT EXISTS(SELECT 1 FROM ingestion_jobs WHERE json_extract(data,'$.previousJobId')=?1 AND json_extract(data,'$.status')<>'undone' AND (json_extract(data,'$.status')<>'cancelled' OR json_array_length(json_extract(data,'$.outcomes'))>0) AND json_extract(data,'$.revision')>0)", [id.to_string()], |r| r.get(0))?;
    if exists {
        return Err(AppError::new(ErrorCode::RevisionConflict));
    }
    Ok(())
}

fn ensure_no_lineage_fork(connection: &Connection, job: Uuid, previous: Uuid) -> Result<()> {
    let exists: bool = connection.query_row("SELECT EXISTS(SELECT 1 FROM ingestion_jobs WHERE id<>?1 AND json_extract(data,'$.previousJobId')=?2 AND json_extract(data,'$.status')<>'undone' AND (json_extract(data,'$.status')<>'cancelled' OR json_array_length(json_extract(data,'$.outcomes'))>0) AND json_extract(data,'$.revision')>0)", params![job.to_string(), previous.to_string()], |r| r.get(0))?;
    if exists {
        return Err(AppError::new(ErrorCode::RevisionConflict));
    }
    Ok(())
}

fn comparison_bytes(item: &ReimportItem) -> usize {
    item.key.len()
        + item.current_title.as_ref().map_or(0, String::len)
        + item.current_body.as_ref().map_or(0, String::len)
        + item
            .prior
            .iter()
            .chain(item.incoming.iter())
            .map(|v| v.title.len() + v.body.len() + v.locator.value.len())
            .sum::<usize>()
}

/// Admission checks use immutable references, never mutable current material heads.
/// Historical operation snapshots may precede the persisted job revision.
pub(super) fn validate_native_job(connection: &Connection, job: &IngestionJob) -> Result<()> {
    use std::collections::HashSet;
    if job.id.is_nil()
        || job.revision > super::super::types::MAX_REVISION as u64
        || !matches!(job.handler_version, 1 | 2)
        || job.extraction.contract_version != job.handler_version
        || (job.handler_version == 2 && job.extraction.format != ExtractionFormat::Json)
        || storage::find(connection, job.source.id)? != job.source
        || job.extraction.records.len() > 10_000
        || (job.extraction.status != ExtractionStatus::Complete
            && !job.extraction.records.is_empty())
    {
        return Err(corrupt());
    }
    let current_data: String = connection.query_row(
        "SELECT data FROM ingestion_jobs WHERE id=?1",
        [job.id.to_string()],
        |r| r.get(0),
    )?;
    let current: IngestionJob = serde_json::from_str(&current_data).map_err(|_| corrupt())?;
    if current.id != job.id
        || current.source != job.source
        || current.extraction != job.extraction
        || current.handler_version != job.handler_version
        || current.revision < job.revision
        || (job.mapping.is_some() && current.mapping != job.mapping)
    {
        return Err(corrupt());
    }
    if let Some(mapping) = &job.mapping {
        if mapping.key_column >= 64
            || mapping.title_column >= 64
            || mapping.body_column >= 64
            || (mapping.json_properties.is_some() && job.handler_version != 2)
        {
            return Err(corrupt());
        }
    } else if !job.outcomes.is_empty() || !job.decisions.is_empty() || job.previous_job_id.is_some()
    {
        return Err(corrupt());
    }
    for record in &job.extraction.records {
        if record.fields.len() > 64
            || record.properties.len() > 64
            || record.duplicate_properties.len() > 64
        {
            return Err(corrupt());
        }
    }
    let revision_owned = |resource: ResourceId, revision: Uuid| -> Result<bool> {
        Ok(connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM resource_revisions WHERE resource_id=?1 AND id=?2)",
            params![resource.to_string(), revision.to_string()],
            |r| r.get(0),
        )?)
    };
    let mut indices = HashSet::new();
    let mut keys = HashSet::new();
    for outcome in &job.outcomes {
        let record = job
            .extraction
            .records
            .get(outcome.index)
            .ok_or_else(corrupt)?;
        let mapping = job.mapping.as_ref().ok_or_else(corrupt)?;
        let (key, title, _) = mapped_fields(record, mapping).ok_or_else(corrupt)?;
        if !indices.insert(outcome.index)
            || !keys.insert(&outcome.key)
            || key.is_empty()
            || key.len() > 1024
            || key != outcome.key
            || title.is_empty()
            || !revision_owned(outcome.resource_id, outcome.document_revision_id)?
            || outcome.before_revision_id.is_some() != outcome.before_title.is_some()
        {
            return Err(corrupt());
        }
        if let Some(revision) = outcome.before_revision_id
            && !revision_owned(outcome.resource_id, revision)?
        {
            return Err(corrupt());
        }
    }
    let mut removed = HashSet::new();
    for decision in &job.decisions {
        match (
            decision.index,
            decision.key.as_deref(),
            decision.disposition,
        ) {
            (Some(index), None, disposition)
                if disposition != IngestionDisposition::RemovedRetained
                    && disposition != IngestionDisposition::Pending =>
            {
                if index >= job.extraction.records.len() || !indices.insert(index) {
                    return Err(corrupt());
                }
            }
            (None, Some(key), IngestionDisposition::RemovedRetained)
                if !key.is_empty() && key.len() <= 1024 && job.previous_job_id.is_some() =>
            {
                if !removed.insert(key) {
                    return Err(corrupt());
                }
            }
            _ => return Err(corrupt()),
        }
    }
    let mut visited = HashSet::from([job.id]);
    let mut previous = job.previous_job_id;
    while let Some(id) = previous {
        if !visited.insert(id) || visited.len() > 257 {
            return Err(corrupt());
        }
        let data: String = connection.query_row(
            "SELECT data FROM ingestion_jobs WHERE id=?1",
            [id.to_string()],
            |r| r.get(0),
        )?;
        let ancestor: IngestionJob = serde_json::from_str(&data).map_err(|_| corrupt())?;
        if ancestor.id != id
            || ancestor.source.resource_id != job.source.resource_id
            || ancestor.mapping != job.mapping
        {
            return Err(corrupt());
        }
        previous = ancestor.previous_job_id;
    }
    Ok(())
}
