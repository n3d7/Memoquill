use super::*;
use crate::{
    document::{BlockContent, Document},
    organization::ResourceId,
};

impl Database {
    pub(in crate::workspace) fn save_document(
        &mut self,
        context: RequestContext,
        request: &SaveDocument,
    ) -> Result<SavedDocument> {
        self.validate_content_write(
            context,
            &request.workspace_id,
            request.contract_version,
            request.expected_revision,
        )?;
        request.document.validate().map_err(|_| invalid())?;
        let hash = fingerprint(
            "document",
            &request.workspace_id,
            request.expected_revision,
            &(request.resource_id, &request.document),
        )?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some(receipt) =
            operations::exact_receipt(&tx, &self.id, &request.operation_id, &hash)?
        {
            return document_outcome(receipt);
        }
        let value = save_revision(&tx, request)?;
        finish(
            tx,
            &self.id,
            &request.operation_id,
            &hash,
            "document",
            OperationOutcome::Document(value.clone()),
        )?;
        Ok(value)
    }
    pub(in crate::workspace) fn read_document(
        &self,
        context: RequestContext,
        id: ResourceId,
        revision: Option<Uuid>,
    ) -> Result<Option<SavedDocument>> {
        context.authorize()?;
        ensure_resource(&self.connection, id, false)?;
        read(&self.connection, id, revision)
    }
}
pub(super) fn ensure_resource(
    connection: &Connection,
    id: ResourceId,
    writable: bool,
) -> Result<()> {
    let archived: Option<bool> = connection
        .query_row(
            "SELECT archived FROM resources WHERE id=?1",
            [id.to_string()],
            |r| r.get(0),
        )
        .optional()?;
    match archived {
        Some(false) => Ok(()),
        Some(true) if !writable => Ok(()),
        _ => Err(invalid()),
    }
}
pub(in crate::workspace) fn read(
    connection: &Connection,
    id: ResourceId,
    revision: Option<Uuid>,
) -> Result<Option<SavedDocument>> {
    let row:Option<(String,i64,String,String)>=match revision {
        Some(revision)=>connection.query_row("SELECT id,revision,document,digest FROM resource_revisions WHERE resource_id=?1 AND id=?2",params![id.to_string(),revision.to_string()],row).optional()?,
        None=>connection.query_row("SELECT r.id,r.revision,r.document,r.digest FROM resource_revisions r JOIN resource_heads h ON h.revision_id=r.id WHERE h.resource_id=?1 AND r.resource_id=h.resource_id",[id.to_string()],row).optional()?,
    };
    row.map(|row| decode_revision(id, row)).transpose()
}
fn decode_revision(id: ResourceId, row: (String, i64, String, String)) -> Result<SavedDocument> {
    let (revision_id, revision, data, digest) = row;
    let document = Document::parse(data.as_bytes()).map_err(|_| corrupt())?;
    if revision <= 0
        || revision > super::super::types::MAX_REVISION
        || document.digest().map_err(|_| corrupt())? != digest
    {
        return Err(corrupt());
    }
    Ok(SavedDocument {
        id: uuid(&revision_id)?,
        resource_id: id,
        revision: integer(revision)?,
        document,
    })
}

fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<(String, i64, String, String)> {
    Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
}
fn insert(tx: &Connection, value: &SavedDocument) -> Result<()> {
    let bytes = value.document.canonical_bytes().map_err(|_| invalid())?;
    let data = String::from_utf8(bytes).map_err(|_| invalid())?;
    tx.execute(
        "INSERT INTO resource_revisions VALUES(?1,?2,?3,?4,?5)",
        params![
            value.id.to_string(),
            value.resource_id.to_string(),
            value.revision as i64,
            data,
            value.document.digest().map_err(|_| invalid())?
        ],
    )?;
    tx.execute("INSERT INTO resource_heads VALUES(?1,?2) ON CONFLICT(resource_id) DO UPDATE SET revision_id=excluded.revision_id",params![value.resource_id.to_string(),value.id.to_string()])?;
    Ok(())
}
fn validate_assets(tx: &Connection, document: &Document) -> Result<()> {
    for block in document.all_blocks() {
        let digest = match &block.content {
            BlockContent::Media { asset_digest, .. } => Some(asset_digest),
            BlockContent::Unsupported {
                source: Some(source),
                ..
            } => Some(&source.asset_digest),
            _ => None,
        };
        if let Some(digest) = digest {
            let exists: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM assets WHERE digest=?1)",
                [digest],
                |r| r.get(0),
            )?;
            if !exists {
                return Err(invalid());
            }
        }
    }
    Ok(())
}
fn document_outcome(receipt: OperationReceipt) -> Result<SavedDocument> {
    match receipt.result {
        ReceiptResult::Applied(OperationOutcome::Document(value)) => {
            value.document.validate().map_err(|_| corrupt())?;
            Ok(value)
        }
        ReceiptResult::Interrupted(code) => Err(AppError::new(code)),
        _ => Err(corrupt()),
    }
}

pub(in crate::workspace) fn save_revision(
    tx: &Connection,
    request: &SaveDocument,
) -> Result<SavedDocument> {
    ensure_resource(tx, request.resource_id, true)?;
    validate_assets(tx, &request.document)?;
    let current = read(tx, request.resource_id, None)?
        .map(|x| x.revision)
        .unwrap_or(0);
    if current != request.expected_revision {
        return Err(conflict());
    }
    let value = SavedDocument {
        id: Uuid::new_v4(),
        resource_id: request.resource_id,
        revision: current + 1,
        document: request.document.clone(),
    };
    insert(tx, &value)?;
    Ok(value)
}
