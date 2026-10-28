use super::{database::Database, error::Result, migrations::checksum, *};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use serde::Serialize;

pub(super) struct OperationRecord {
    pub hash: String,
    pub state: String,
    pub result: Option<String>,
}

impl Database {
    pub(super) fn save_preferences(
        &mut self,
        context: RequestContext,
        request: &SavePreferences,
    ) -> Result<PreferencesSnapshot> {
        self.authorize_workspace(context, &request.workspace_id)?;
        validate_preferences_request(request)?;
        let hash = request_hash("preferences", request)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let snapshot = change_preferences(&tx, &self.id, request, &hash)?;
        durable_commit(tx)?;
        Ok(snapshot)
    }
    pub(super) fn operation_receipt(
        &self,
        context: RequestContext,
        id: &OperationId,
    ) -> Result<Option<OperationReceipt>> {
        context.authorize()?;
        record(&self.connection, &self.id, id)?
            .map(|r| decode(id, r))
            .transpose()
    }
    pub(super) fn authorize_workspace(
        &self,
        context: RequestContext,
        id: &WorkspaceId,
    ) -> Result<()> {
        context.authorize()?;
        if id != &self.id {
            return Err(AppError::new(ErrorCode::PermissionDenied));
        }
        if self.recovery.blocks_writes() {
            return Err(AppError::new(ErrorCode::RecoveryRequired));
        }
        Ok(())
    }
}

fn validate_preferences_request(request: &SavePreferences) -> Result<()> {
    if request.contract_version != 1 || !(0..MAX_REVISION).contains(&request.expected_revision) {
        return Err(AppError::new(ErrorCode::InvalidRequest));
    }
    Ok(())
}
fn change_preferences(
    tx: &Transaction<'_>,
    workspace: &WorkspaceId,
    request: &SavePreferences,
    hash: &str,
) -> Result<PreferencesSnapshot> {
    if let Some(receipt) = exact_receipt(tx, workspace, &request.operation_id, hash)? {
        return preferences_outcome(receipt, workspace);
    }
    let revision: i64 = tx.query_row(
        "SELECT revision FROM preferences WHERE workspace_id=?1",
        [workspace.as_str()],
        |r| r.get(0),
    )?;
    if revision != request.expected_revision {
        return Err(AppError::new(ErrorCode::RevisionConflict));
    }
    let snapshot = PreferencesSnapshot {
        workspace_id: workspace.clone(),
        revision: revision + 1,
        preferences: request.preferences.clone(),
    };
    tx.execute(
        "UPDATE preferences SET revision=?1,data=?2 WHERE workspace_id=?3",
        params![
            snapshot.revision,
            json(&snapshot.preferences)?,
            workspace.as_str()
        ],
    )?;
    let receipt = applied(
        request.operation_id.clone(),
        OperationOutcome::Preferences(snapshot.clone()),
    );
    insert_applied(tx, workspace, &receipt, hash, "preferences")?;
    Ok(snapshot)
}

pub(super) fn request_hash(kind: &str, value: &impl Serialize) -> Result<String> {
    Ok(checksum(
        &serde_json::to_vec(&(1, kind, value))
            .map_err(|_| AppError::new(ErrorCode::InvalidRequest))?,
    ))
}
pub(super) fn json(value: &impl Serialize) -> Result<String> {
    serde_json::to_string(value).map_err(|_| AppError::new(ErrorCode::InvalidRequest))
}
pub(super) fn applied(id: OperationId, outcome: OperationOutcome) -> OperationReceipt {
    OperationReceipt {
        operation_id: id,
        result: ReceiptResult::Applied(outcome),
    }
}
pub(super) fn record(
    connection: &Connection,
    workspace: &WorkspaceId,
    id: &OperationId,
) -> Result<Option<OperationRecord>> {
    Ok(connection.query_row("SELECT request_hash,state,result FROM operations WHERE id=?1 AND workspace_id=?2 AND principal='local-ui'",params![id.as_str(),workspace.as_str()],|r|Ok(OperationRecord{hash:r.get(0)?,state:r.get(1)?,result:r.get(2)?})).optional()?)
}
pub(super) fn exact_receipt(
    connection: &Connection,
    workspace: &WorkspaceId,
    id: &OperationId,
    hash: &str,
) -> Result<Option<OperationReceipt>> {
    let Some(row) = record(connection, workspace, id)? else {
        return Ok(None);
    };
    if row.hash != hash {
        return Err(AppError::new(ErrorCode::IdempotencyConflict));
    }
    Ok(Some(decode(id, row)?))
}
fn decode(id: &OperationId, row: OperationRecord) -> Result<OperationReceipt> {
    if row.state == "prepared" && row.result.is_none() {
        return Ok(OperationReceipt {
            operation_id: id.clone(),
            result: ReceiptResult::Pending,
        });
    }
    terminal_receipt(id, row)
}
fn terminal_receipt(id: &OperationId, row: OperationRecord) -> Result<OperationReceipt> {
    let value = row
        .result
        .ok_or(AppError::new(ErrorCode::RecoveryRequired))?;
    let receipt: OperationReceipt =
        serde_json::from_str(&value).map_err(|_| AppError::new(ErrorCode::RecoveryRequired))?;
    let state = match &receipt.result {
        ReceiptResult::Applied(_) => "applied",
        ReceiptResult::Interrupted(_) => "interrupted",
        ReceiptResult::Pending => "prepared",
    };
    if receipt.operation_id != *id || row.state != state {
        return Err(AppError::new(ErrorCode::RecoveryRequired));
    }
    Ok(receipt)
}
fn preferences_outcome(
    receipt: OperationReceipt,
    workspace: &WorkspaceId,
) -> Result<PreferencesSnapshot> {
    match receipt.result {
        ReceiptResult::Applied(OperationOutcome::Preferences(value))
            if &value.workspace_id == workspace && value.revision > 0 =>
        {
            Ok(value)
        }
        ReceiptResult::Interrupted(code) => Err(AppError::new(code)),
        _ => Err(AppError::new(ErrorCode::RecoveryRequired)),
    }
}
pub(super) fn insert_applied(
    connection: &Connection,
    workspace: &WorkspaceId,
    receipt: &OperationReceipt,
    hash: &str,
    kind: &str,
) -> Result<()> {
    connection.execute("INSERT INTO operations(id,workspace_id,principal,request_hash,kind,state,result) VALUES (?1,?2,'local-ui',?3,?4,'applied',?5)",params![receipt.operation_id.as_str(),workspace.as_str(),hash,kind,json(receipt)?])?;
    Ok(())
}
pub(super) fn durable_commit(transaction: Transaction<'_>) -> Result<()> {
    transaction
        .commit()
        .map_err(|_| AppError::new(ErrorCode::OutcomeUnconfirmed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revision_boundary_preserves_exact_javascript_integer_round_trips() {
        let temp = tempfile::tempdir().unwrap();
        let mut db = Database::open(&temp.path().join("workspace")).unwrap();
        let mut request = save_request(&db);
        request.expected_revision = 9_007_199_254_740_991;
        assert_eq!(
            db.save_preferences(RequestContext::local_ui(), &request)
                .unwrap_err()
                .code,
            ErrorCode::InvalidRequest
        );
        db.connection
            .execute(
                "UPDATE preferences SET revision=?1",
                [9_007_199_254_740_992_i64],
            )
            .unwrap();
        assert_eq!(
            db.preferences().unwrap_err().code,
            ErrorCode::RecoveryRequired
        );
    }

    pub(super) fn save_request(db: &Database) -> SavePreferences {
        SavePreferences {
            contract_version: 1,
            workspace_id: db.id.clone(),
            operation_id: OperationId::generate(),
            expected_revision: 0,
            preferences: ReadingPreferences {
                theme: Theme::Dark,
                text_scale: TextScale::try_from(200).unwrap(),
            },
        }
    }

    #[test]
    fn applied_preferences_survive_restart_and_exact_retry_returns_one_receipt() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        let mut db = Database::open(&root).unwrap();
        let request = save_request(&db);
        let result = db
            .save_preferences(RequestContext::local_ui(), &request)
            .unwrap();
        assert_eq!(result.revision, 1);
        assert_eq!(result.preferences, request.preferences);
        drop(db); // A lost response after commit must not repeat the change.
        let mut reopened = Database::open(&root).unwrap();
        assert_eq!(
            reopened
                .save_preferences(RequestContext::local_ui(), &request)
                .unwrap(),
            result
        );
        assert_eq!(reopened.preferences().unwrap(), result);
        let receipt = reopened
            .operation_receipt(RequestContext::local_ui(), &request.operation_id)
            .unwrap()
            .unwrap();
        assert_eq!(
            receipt.result,
            ReceiptResult::Applied(OperationOutcome::Preferences(result))
        );
        assert_eq!(
            reopened
                .connection
                .query_row("SELECT count(*) FROM operations", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            1
        );
    }

    #[test]
    fn stale_revision_changed_key_payload_and_untrusted_context_do_not_write() {
        let temp = tempfile::tempdir().unwrap();
        let mut db = Database::open(&temp.path().join("workspace")).unwrap();
        let mut request = save_request(&db);
        assert_eq!(
            db.save_preferences(RequestContext::unverified_external(), &request)
                .unwrap_err()
                .code,
            ErrorCode::PermissionDenied
        );
        assert_eq!(db.preferences().unwrap().revision, 0);
        db.save_preferences(RequestContext::local_ui(), &request)
            .unwrap();
        request.preferences.theme = Theme::Light;
        assert_eq!(
            db.save_preferences(RequestContext::local_ui(), &request)
                .unwrap_err()
                .code,
            ErrorCode::IdempotencyConflict
        );
        request.operation_id = OperationId::generate();
        assert_eq!(
            db.save_preferences(RequestContext::local_ui(), &request)
                .unwrap_err()
                .code,
            ErrorCode::RevisionConflict
        );
        request.workspace_id = WorkspaceId::generate();
        assert_eq!(
            db.save_preferences(RequestContext::local_ui(), &request)
                .unwrap_err()
                .code,
            ErrorCode::PermissionDenied
        );
        assert_eq!(db.preferences().unwrap().revision, 1);
    }
    #[test]
    fn v1_fingerprint_remains_compatible_with_persisted_receipts() {
        let request:SavePreferences=serde_json::from_value(serde_json::json!({"contractVersion":1,
            "workspaceId":"4d9606cd-1b54-4fbe-8d21-414bc20d792a","operationId":"9bac6ad2-596d-498b-bc2f-48585055fa84",
            "expectedRevision":3,"preferences":{"theme":"dark","textScale":125}})).unwrap();
        assert_eq!(
            request_hash("preferences", &request).unwrap(),
            "1afe3d4174e59b997c4dcbb261d76d4b8436e9b3b6a846a3a0685a136ec3e018"
        );
    }
    #[test]
    fn pending_and_corrupt_receipt_states_cannot_impersonate_applied_operations() {
        let id = OperationId::generate();
        let other = OperationId::generate();
        let row = |state: &str, result: Option<String>| OperationRecord {
            hash: "hash".into(),
            state: state.into(),
            result,
        };
        assert_eq!(
            decode(&id, row("prepared", None)).unwrap().result,
            ReceiptResult::Pending
        );
        assert!(decode(&id, row("applied", None)).is_err());
        assert!(
            decode(
                &id,
                row(
                    "prepared",
                    Some(
                        json(&applied(
                            id.clone(),
                            OperationOutcome::Asset(AssetInfo {
                                digest: AssetDigest::try_from("a".repeat(64)).unwrap(),
                                byte_length: 0
                            })
                        ))
                        .unwrap()
                    )
                )
            )
            .is_err()
        );
        let receipt = OperationReceipt {
            operation_id: other,
            result: ReceiptResult::Interrupted(ErrorCode::StorageFull),
        };
        assert!(decode(&id, row("interrupted", Some(json(&receipt).unwrap()))).is_err());
        let receipt = OperationReceipt {
            operation_id: id.clone(),
            result: ReceiptResult::Interrupted(ErrorCode::StorageFull),
        };
        assert!(decode(&id, row("applied", Some(json(&receipt).unwrap()))).is_err());
        assert!(decode(&id, row("applied", Some("invalid JSON".into()))).is_err());
    }
    #[test]
    fn applied_preference_receipts_require_the_right_workspace_and_positive_revision() {
        let workspace = WorkspaceId::generate();
        for (id, revision) in [(WorkspaceId::generate(), 1), (workspace.clone(), 0)] {
            let receipt = applied(
                OperationId::generate(),
                OperationOutcome::Preferences(PreferencesSnapshot {
                    workspace_id: id,
                    revision,
                    preferences: ReadingPreferences::default(),
                }),
            );
            assert_eq!(
                preferences_outcome(receipt, &workspace).unwrap_err().code,
                ErrorCode::RecoveryRequired
            );
        }
    }
}
