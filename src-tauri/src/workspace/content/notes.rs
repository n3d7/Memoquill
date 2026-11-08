use super::*;
use crate::{
    document::{ResourceRevision, utf16_range},
    organization::ResourceId,
};

impl Database {
    pub(in crate::workspace) fn save_annotation(
        &mut self,
        context: RequestContext,
        request: &SaveAnnotation,
    ) -> Result<Annotation> {
        self.validate_content_write(
            context,
            &request.workspace_id,
            request.contract_version,
            request.expected_revision,
        )?;
        validate_note(&request.annotation)?;
        let hash = fingerprint(
            "annotation",
            &request.workspace_id,
            request.expected_revision,
            &request.annotation,
        )?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some(receipt) =
            operations::exact_receipt(&tx, &self.id, &request.operation_id, &hash)?
        {
            return annotation_outcome(receipt);
        }
        let value = save_note_content(&tx, request)?;
        finish(
            tx,
            &self.id,
            &request.operation_id,
            &hash,
            "annotation",
            OperationOutcome::Annotation(Box::new(value.clone())),
        )?;
        Ok(value)
    }
    pub(in crate::workspace) fn annotations(
        &self,
        context: RequestContext,
        id: ResourceId,
    ) -> Result<Vec<Annotation>> {
        context.authorize()?;
        documents::ensure_resource(&self.connection, id, false)?;
        let rows = self.connection.prepare("SELECT id,revision_id,revision,anchor,body FROM annotations WHERE resource_id=?1 ORDER BY id")?
            .query_map([id.to_string()], annotation_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        rows.into_iter()
            .map(|row| decode_annotation(&self.connection, id, row))
            .collect()
    }
    pub(in crate::workspace) fn save_reading_position(
        &mut self,
        context: RequestContext,
        request: &SaveReadingPosition,
    ) -> Result<ReadingPosition> {
        self.validate_content_write(
            context,
            &request.workspace_id,
            request.contract_version,
            request.expected_revision,
        )?;
        let hash = fingerprint(
            "reading_position",
            &request.workspace_id,
            request.expected_revision,
            &request.position,
        )?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some(receipt) =
            operations::exact_receipt(&tx, &self.id, &request.operation_id, &hash)?
        {
            return position_outcome(receipt);
        }
        let value = save_position_content(&tx, request)?;
        finish(
            tx,
            &self.id,
            &request.operation_id,
            &hash,
            "reading_position",
            OperationOutcome::ReadingPosition(value.clone()),
        )?;
        Ok(value)
    }
    pub(in crate::workspace) fn reading_position(
        &self,
        context: RequestContext,
        id: ResourceId,
    ) -> Result<Option<ReadingPosition>> {
        context.authorize()?;
        documents::ensure_resource(&self.connection, id, false)?;
        let row:Option<(String,String,u32,i64)>=self.connection.query_row("SELECT revision_id,block_id,offset_utf16,revision FROM reading_positions WHERE resource_id=?1",[id.to_string()],|r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional()?;
        row.map(|row| decode_position(&self.connection, id, row))
            .transpose()
    }
}
fn validate_note(value: &Annotation) -> Result<()> {
    if value.id.is_nil()
        || value.body.len() > 65536
        || value.body.contains('\0')
        || value.anchor.resource_id != value.resource_id.as_uuid()
        || value.anchor.revision_id != value.revision_id
    {
        return Err(invalid());
    }
    Ok(())
}
fn validate_note_reference(connection: &Connection, value: &Annotation) -> Result<()> {
    validate_note(value)?;
    let source = documents::read(connection, value.resource_id, Some(value.revision_id))?
        .ok_or_else(invalid)?;
    let revision = ResourceRevision::from_record(
        source.resource_id.as_uuid(),
        source.id,
        source.revision,
        source.document,
    )
    .map_err(|_| invalid())?;
    value.anchor.validate(&revision).map_err(|_| invalid())
}
fn validate_position(tx: &Connection, value: &ReadingPosition) -> Result<()> {
    let source =
        documents::read(tx, value.resource_id, Some(value.revision_id))?.ok_or_else(invalid)?;
    let block = source
        .document
        .all_blocks()
        .into_iter()
        .find(|b| b.id == value.block_id)
        .ok_or_else(invalid)?;
    utf16_range(
        &block.text(),
        value.offset_utf16 as usize,
        value.offset_utf16 as usize,
    )
    .map_err(|_| invalid())?;
    Ok(())
}
fn annotation_outcome(receipt: OperationReceipt) -> Result<Annotation> {
    match receipt.result {
        ReceiptResult::Applied(OperationOutcome::Annotation(value)) => Ok(*value),
        ReceiptResult::Interrupted(code) => Err(AppError::new(code)),
        _ => Err(corrupt()),
    }
}
fn position_outcome(receipt: OperationReceipt) -> Result<ReadingPosition> {
    match receipt.result {
        ReceiptResult::Applied(OperationOutcome::ReadingPosition(value)) => Ok(value),
        ReceiptResult::Interrupted(code) => Err(AppError::new(code)),
        _ => Err(corrupt()),
    }
}

fn save_note_content(tx: &Connection, request: &SaveAnnotation) -> Result<Annotation> {
    let annotation = &request.annotation;
    documents::ensure_resource(tx, annotation.resource_id, true)?;
    validate_note_reference(tx, annotation)?;
    let current = note_revision(tx, annotation)?;
    if integer(current)? != request.expected_revision {
        return Err(conflict());
    }
    let mut value = annotation.clone();
    value.revision = integer(current)? + 1;
    tx.execute("INSERT INTO annotations VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(id) DO UPDATE SET revision=excluded.revision,anchor=excluded.anchor,body=excluded.body",params![value.id.to_string(),value.resource_id.to_string(),value.revision_id.to_string(),value.revision as i64,operations::json(&value.anchor)?,value.body])?;
    Ok(value)
}

fn save_position_content(
    tx: &Connection,
    request: &SaveReadingPosition,
) -> Result<ReadingPosition> {
    documents::ensure_resource(tx, request.position.resource_id, true)?;
    validate_position(tx, &request.position)?;
    let current: i64 = tx
        .query_row(
            "SELECT revision FROM reading_positions WHERE resource_id=?1",
            [request.position.resource_id.to_string()],
            |r| r.get(0),
        )
        .optional()?
        .unwrap_or(0);
    if integer(current)? != request.expected_revision {
        return Err(conflict());
    }
    let mut value = request.position.clone();
    value.revision = integer(current)? + 1;
    tx.execute("INSERT INTO reading_positions VALUES(?1,?2,?3,?4,?5) ON CONFLICT(resource_id) DO UPDATE SET revision_id=excluded.revision_id,block_id=excluded.block_id,offset_utf16=excluded.offset_utf16,revision=excluded.revision",params![value.resource_id.to_string(),value.revision_id.to_string(),value.block_id.to_string(),value.offset_utf16,value.revision as i64])?;
    Ok(value)
}

fn note_revision(tx: &Connection, annotation: &Annotation) -> Result<i64> {
    let current: Option<(i64, String, String)> = tx
        .query_row(
            "SELECT revision,resource_id,revision_id FROM annotations WHERE id=?1",
            [annotation.id.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    if let Some((_, resource, revision)) = &current
        && (resource != &annotation.resource_id.to_string()
            || revision != &annotation.revision_id.to_string())
    {
        return Err(invalid());
    }
    let current = current.map(|x| x.0).unwrap_or(0);
    Ok(current)
}

type AnnotationRow = (String, String, i64, String, String);
fn decode_annotation(
    connection: &Connection,
    id: ResourceId,
    row: AnnotationRow,
) -> Result<Annotation> {
    let (key, revision_id, revision, anchor, body) = row;
    let value = Annotation {
        id: uuid(&key)?,
        resource_id: id,
        revision_id: uuid(&revision_id)?,
        revision: integer(revision)?,
        anchor: serde_json::from_str(&anchor).map_err(|_| corrupt())?,
        body,
    };
    if value.revision == 0 {
        return Err(corrupt());
    }
    validate_note_reference(connection, &value).map_err(|_| corrupt())?;
    Ok(value)
}

fn decode_position(
    connection: &Connection,
    id: ResourceId,
    row: (String, String, u32, i64),
) -> Result<ReadingPosition> {
    let (revision, block, offset, version) = row;
    let value = ReadingPosition {
        resource_id: id,
        revision_id: uuid(&revision)?,
        block_id: uuid(&block)?,
        offset_utf16: offset,
        revision: integer(version)?,
    };
    if value.revision == 0 {
        return Err(corrupt());
    }
    validate_position(connection, &value).map_err(|_| corrupt())?;
    Ok(value)
}

fn annotation_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AnnotationRow> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
    ))
}
