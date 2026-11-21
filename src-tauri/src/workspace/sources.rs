//! Immutable managed originals, separate from learning-content revisions.
use super::{database::Database, error::Result, *};
use crate::organization::ResourceId;
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use uuid::Uuid;

mod extraction;
mod ingestion;
mod preview;
mod storage;
mod types;
mod validation;
pub use ingestion::{
    ApplyIngestionJob, ApplyReimportJob, CancelIngestionJob, CreateIngestionJob, IngestionDecision,
    IngestionDisposition, IngestionJob, IngestionJobSummary, IngestionMapping, IngestionOutcome,
    IngestionStatus, JsonPropertyMapping, ReimportItem, ReimportPreview, ReimportResolution,
    ReimportStatus, ReviewIngestionJob, UndoIngestionJob, UndoPreview,
};
pub use types::*;
#[cfg(test)]
mod tests;

impl Database {
    pub(super) fn sources(&self, context: RequestContext, id: ResourceId) -> Result<SourceList> {
        context.authorize()?;
        storage::resource(&self.connection, id, false)?;
        storage::list(&self.connection, id)
    }
    pub(super) fn save_source(
        &mut self,
        context: RequestContext,
        request: &SaveSource,
    ) -> Result<OriginalSource> {
        self.authorize_workspace(context, &request.workspace_id)?;
        validate_request(request)?;
        let hash = operations::request_hash(
            "source",
            &(
                request.workspace_id.clone(),
                request.expected_revision,
                &request.source,
            ),
        )?;
        let root = self.owner.root().to_path_buf();
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some(receipt) =
            operations::exact_receipt(&tx, &self.id, &request.operation_id, &hash)?
        {
            return source_outcome(receipt, request);
        }
        let value = storage::insert(&tx, &root, request)?;
        commit_source(tx, &self.id, &request.operation_id, &hash, &value)?;
        Ok(value)
    }
    pub(super) fn original_copy(
        &self,
        context: RequestContext,
        id: uuid::Uuid,
    ) -> Result<(String, Vec<u8>)> {
        context.authorize()?;
        let source = storage::find(&self.connection, id)?;
        let info = AssetInfo {
            digest: source.digest,
            byte_length: source.byte_length,
        };
        if info.byte_length > 64 * 1024 * 1024 {
            return Err(AppError::new(ErrorCode::BudgetExceeded));
        }
        assets::verify(self.owner.root(), &info)?;
        let bytes = preview::read_bytes(self.owner.root(), &info, 64 * 1024 * 1024)?;
        use sha2::{Digest, Sha256};
        if format!("{:x}", Sha256::digest(&bytes)) != info.digest.as_str() {
            return Err(AppError::new(ErrorCode::RecoveryRequired));
        }
        Ok((source.filename, bytes))
    }
    #[cfg(target_os = "linux")]
    pub(super) fn mcp_lesson(
        &self,
        context: RequestContext,
        unit: &str,
        include_sources: bool,
    ) -> Result<Option<super::mcp_reads::PublishedLesson>> {
        let Some(saved) = self.lesson_read(context, unit, None)? else {
            return Ok(None);
        };
        let mut source_revisions = std::collections::BTreeSet::new();
        if include_sources {
            for revision in published_source_revisions(&saved.composition) {
                if self.mcp_active_source(unit, revision)?.is_some() {
                    source_revisions.insert(revision.to_owned());
                }
            }
        }
        Ok(Some(super::mcp_reads::PublishedLesson {
            saved,
            source_revisions: source_revisions.into_iter().collect(),
        }))
    }

    #[cfg(target_os = "linux")]
    pub(super) fn mcp_active_source(
        &self,
        unit: &str,
        revision: &str,
    ) -> Result<Option<(SourceBindingRevision, OriginalSource)>> {
        let Some(binding) = self.published_lesson_source(unit, revision)? else {
            return Ok(None);
        };
        let id = Uuid::parse_str(&binding.source_id).map_err(|_| corrupt())?;
        let source = storage::find(&self.connection, id)?;
        if source.digest.as_str() != binding.asset_digest {
            return Err(corrupt());
        }
        match storage::resource(&self.connection, source.resource_id, true) {
            Ok(()) => Ok(Some((binding, source))),
            Err(error) if error.code == ErrorCode::InvalidRequest => Ok(None),
            Err(error) => Err(error),
        }
    }

    #[cfg(target_os = "linux")]
    pub(super) fn mcp_bound_source(
        &self,
        context: RequestContext,
        unit: &str,
        revision: &str,
        original_offset: Option<u64>,
    ) -> Result<super::mcp_reads::BoundSource> {
        use super::mcp_reads::BoundSource;
        let denied = || AppError::new(ErrorCode::PermissionDenied);
        let lesson = self.lesson_read(context, unit, None)?.ok_or_else(denied)?;
        if !published_source_revisions(&lesson.composition).any(|id| id == revision) {
            return Err(denied());
        }
        let (binding, source) = self.mcp_active_source(unit, revision)?.ok_or_else(denied)?;
        if let Some(offset) = original_offset {
            let root = self.owner.root();
            let info = AssetInfo {
                digest: source.digest,
                byte_length: source.byte_length,
            };
            let bytes = preview::verified_range(root, &info, offset, 16 * 1024)?;
            Ok(BoundSource::Original {
                binding,
                byte_length: info.byte_length,
                bytes,
            })
        } else {
            Ok(BoundSource::Extracted {
                binding,
                preview: Box::new(preview::read(self.owner.root(), source)?),
            })
        }
    }

    pub(super) fn source_preview(
        &self,
        context: RequestContext,
        id: Uuid,
    ) -> Result<SourcePreview> {
        context.authorize()?;
        if id.is_nil() {
            return Err(invalid());
        }
        let source = storage::find(&self.connection, id)?;
        preview::read(self.owner.root(), source)
    }
}

#[cfg(target_os = "linux")]
fn published_source_revisions(composition: &Composition) -> impl Iterator<Item = &str> {
    composition
        .bindings
        .iter()
        .filter(|binding| binding.target_kind == "source")
        .map(|binding| binding.revision_id.as_str())
}

fn commit_source(
    tx: rusqlite::Transaction<'_>,
    workspace: &WorkspaceId,
    operation: &OperationId,
    hash: &str,
    source: &OriginalSource,
) -> Result<()> {
    let receipt = operations::applied(
        operation.clone(),
        OperationOutcome::Source(Box::new(source.clone())),
    );
    operations::insert_applied(&tx, workspace, &receipt, hash, "asset")?;
    operations::durable_commit(tx)
}

fn invalid() -> AppError {
    AppError::new(ErrorCode::InvalidRequest)
}
fn corrupt() -> AppError {
    AppError::new(ErrorCode::RecoveryRequired)
}

fn validate_request(request: &SaveSource) -> Result<()> {
    if request.contract_version != 1
        || request.expected_revision >= super::types::MAX_REVISION as u64
    {
        return Err(invalid());
    }
    validation::new_source(&request.source)
}

fn source_outcome(receipt: OperationReceipt, request: &SaveSource) -> Result<OriginalSource> {
    match receipt.result {
        ReceiptResult::Applied(OperationOutcome::Source(value)) => {
            validation::original(&value).map_err(|_| corrupt())?;
            if storage::metadata(&value) != request.source
                || value.revision != request.expected_revision + 1
            {
                return Err(corrupt());
            }
            Ok(*value)
        }
        ReceiptResult::Interrupted(code) => Err(AppError::new(code)),
        _ => Err(corrupt()),
    }
}

pub(super) fn validate_native_data(connection: &Connection) -> Result<()> {
    storage::validate_native_data(connection)
}

pub(super) fn validate_native_outcome(
    connection: &Connection,
    outcome: &OperationOutcome,
) -> Result<()> {
    match outcome {
        OperationOutcome::Source(value) => {
            validation::original(value)?;
            if storage::find(connection, value.id)? != **value {
                return Err(corrupt());
            }
        }
        OperationOutcome::IngestionJob(value) => {
            ingestion::validate_native_job(connection, value)?;
        }
        _ => return Err(corrupt()),
    }
    Ok(())
}

pub(super) fn validate_native_job(connection: &Connection, job: &IngestionJob) -> Result<()> {
    ingestion::validate_native_job(connection, job)
}
