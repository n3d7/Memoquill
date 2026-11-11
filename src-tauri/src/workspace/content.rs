//! Typed durable content operations. SQL and caller paths never cross this interface.
use super::{database::Database, error::Result, *};
use crate::organization;
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use uuid::Uuid;
pub(super) mod documents;
mod notes;
mod organization_store;
#[cfg(test)]
mod persistence_tests;
mod types;
pub use types::*;

/// Read-only package admission: reuse the same organization/document invariants
/// as runtime reads after the package reader has verified the exact trusted DDL.
pub(super) fn validate_native_data(connection: &Connection, workspace: &WorkspaceId) -> Result<()> {
    organization_store::load_connection(connection, workspace)?;
    let mut query = connection.prepare("SELECT resource_id,id FROM resource_revisions")?;
    let mut rows = query.query([])?;
    while let Some(row) = rows.next()? {
        let resource: String = row.get(0)?;
        let revision: String = row.get(1)?;
        let resource =
            serde_json::from_value(serde_json::Value::String(resource)).map_err(|_| corrupt())?;
        let revision = uuid(&revision)?;
        documents::read(connection, resource, Some(revision))?.ok_or_else(corrupt)?;
    }
    Ok(())
}

impl Database {
    pub(super) fn library(&self, context: RequestContext) -> Result<organization::Snapshot> {
        context.authorize()?;
        organization_store::load(self)
    }
    pub(super) fn apply_content(
        &mut self,
        context: RequestContext,
        request: &ApplyContent,
    ) -> Result<organization::Snapshot> {
        self.validate_content_write(
            context,
            &request.workspace_id,
            request.contract_version,
            request.expected_revision,
        )?;
        let hash = fingerprint(
            "organization",
            &request.workspace_id,
            request.expected_revision,
            &request.command,
        )?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some(receipt) =
            operations::exact_receipt(&tx, &self.id, &request.operation_id, &hash)?
        {
            return organization_outcome(receipt);
        }
        let result = apply_command(&tx, &self.id, request)?;
        finish(
            tx,
            &self.id,
            &request.operation_id,
            &hash,
            "organization",
            OperationOutcome::Organization(Box::new(result.clone())),
        )?;
        Ok(result)
    }
}

impl Database {
    fn validate_content_write(
        &self,
        context: RequestContext,
        workspace: &WorkspaceId,
        contract: u32,
        revision: u64,
    ) -> Result<()> {
        self.authorize_workspace(context, workspace)?;
        version(contract, revision)
    }
}
pub(super) fn apply_command(
    tx: &Connection,
    workspace: &WorkspaceId,
    request: &ApplyContent,
) -> Result<organization::Snapshot> {
    // The writer owns this connection, so the read and all invariant checks observe this transaction.
    let mut value = organization::Organization::from_snapshot(organization_store::load_connection(
        tx, workspace,
    )?)
    .map_err(|_| corrupt())?;
    value
        .apply(
            uuid(workspace.as_str())?,
            request.expected_revision,
            request.command.clone(),
        )
        .map_err(domain_error)?;
    let result = value.snapshot().clone();
    organization_store::save(tx, &result)?;
    Ok(result)
}

fn invalid() -> AppError {
    AppError::new(ErrorCode::InvalidRequest)
}
fn corrupt() -> AppError {
    AppError::new(ErrorCode::RecoveryRequired)
}
fn integer(value: i64) -> Result<u64> {
    if !(0..=super::types::MAX_REVISION).contains(&value) {
        return Err(corrupt());
    }
    Ok(value as u64)
}
fn conflict() -> AppError {
    AppError::new(ErrorCode::RevisionConflict)
}
fn uuid(value: &str) -> Result<Uuid> {
    let value = Uuid::parse_str(value).map_err(|_| corrupt())?;
    if value.is_nil() {
        return Err(corrupt());
    }
    Ok(value)
}

fn version(value: u32, revision: u64) -> Result<()> {
    if value != 1 || revision >= super::types::MAX_REVISION as u64 {
        return Err(invalid());
    }
    Ok(())
}
fn domain_error(value: organization::DomainError) -> AppError {
    match value {
        organization::DomainError::PermissionDenied => AppError::new(ErrorCode::PermissionDenied),
        organization::DomainError::RevisionConflict => conflict(),
        _ => invalid(),
    }
}
fn fingerprint(
    kind: &str,
    workspace: &WorkspaceId,
    revision: u64,
    value: &impl serde::Serialize,
) -> Result<String> {
    // Value objects use sorted JSON keys; the retry identity itself is deliberately excluded.
    let value =
        serde_json::to_value((1, kind, workspace, revision, value)).map_err(|_| invalid())?;
    Ok(migrations::checksum(
        &serde_json::to_vec(&value).map_err(|_| invalid())?,
    ))
}
fn finish(
    tx: rusqlite::Transaction<'_>,
    workspace: &WorkspaceId,
    id: &OperationId,
    hash: &str,
    kind: &str,
    outcome: OperationOutcome,
) -> Result<()> {
    operations::insert_applied(
        &tx,
        workspace,
        &operations::applied(id.clone(), outcome),
        hash,
        kind,
    )?;
    operations::durable_commit(tx)
}
fn organization_outcome(receipt: OperationReceipt) -> Result<organization::Snapshot> {
    match receipt.result {
        ReceiptResult::Applied(OperationOutcome::Organization(value)) => {
            organization::Organization::from_snapshot((*value).clone()).map_err(|_| corrupt())?;
            Ok(*value)
        }
        ReceiptResult::Interrupted(code) => Err(AppError::new(code)),
        _ => Err(corrupt()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fresh_library_has_workspace_identity_and_no_subject_template() {
        tauri::async_runtime::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            let service = ApplicationService::start(temp.path().join("workspace")).unwrap();
            let status = service.status(RequestContext::local_ui()).await.unwrap();
            let library = service.library(RequestContext::local_ui()).await.unwrap();
            assert_eq!(
                library.workspace_id.to_string(),
                status.preferences.workspace_id.as_str()
            );
            assert_eq!(library.revision, 0);
            assert!(library.subjects.is_empty());
            assert_eq!(
                service
                    .library(RequestContext::unverified_external())
                    .await
                    .unwrap_err()
                    .code,
                ErrorCode::PermissionDenied
            );
            service.shutdown().await.unwrap();
        });
    }
}
