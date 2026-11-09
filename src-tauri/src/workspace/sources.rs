//! Immutable managed originals, separate from learning-content revisions.
use super::{database::Database, error::Result, *};
use crate::organization::ResourceId;
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use uuid::Uuid;

mod extraction;
mod preview;
mod storage;
mod types;
mod validation;
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
