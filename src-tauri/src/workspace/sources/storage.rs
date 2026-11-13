use super::*;
use std::path::Path;

const PROJECTION: &str = "json_object('id',id,'resourceId',resource_id,'digest',digest,'filename',filename,'mediaType',media_type,'attribution',attribution,'license',license,'originUrl',origin_url,'revision',revision,'byteLength',byte_length)";

pub(super) fn resource(connection: &Connection, id: ResourceId, write: bool) -> Result<()> {
    let archived: Option<bool> = connection
        .query_row(
            "SELECT archived FROM resources WHERE id=?1",
            [id.to_string()],
            |r| r.get(0),
        )
        .optional()?;
    match archived {
        Some(false) => Ok(()),
        Some(true) if !write => Ok(()),
        _ => Err(invalid()),
    }
}

fn revision(connection: &Connection, id: ResourceId) -> Result<u64> {
    let value: i64 = connection
        .query_row(
            "SELECT revision FROM source_revisions WHERE resource_id=?1",
            [id.to_string()],
            |r| r.get(0),
        )
        .optional()?
        .unwrap_or(0);
    if !(0..=super::super::types::MAX_REVISION).contains(&value) {
        return Err(corrupt());
    }
    Ok(value as u64)
}

pub(super) fn list(connection: &Connection, id: ResourceId) -> Result<SourceList> {
    let revision = revision(connection, id)?;
    let records = records(connection, id)?;
    let mut sources = Vec::with_capacity(records.len());
    for (index, record) in records.into_iter().enumerate() {
        let source = decode(connection, &record)?;
        if source.resource_id != id || source.revision != index as u64 + 1 {
            return Err(corrupt());
        }
        sources.push(source);
    }
    if sources.len() as u64 != revision {
        return Err(corrupt());
    }
    Ok(SourceList { revision, sources })
}

fn records(connection: &Connection, id: ResourceId) -> Result<Vec<String>> {
    let query =
        format!("SELECT {PROJECTION} FROM original_sources WHERE resource_id=?1 ORDER BY revision");
    let mut statement = connection.prepare(&query)?;
    let records = statement
        .query_map([id.to_string()], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(records)
}

pub(super) fn find(connection: &Connection, id: Uuid) -> Result<OriginalSource> {
    let query = format!("SELECT {PROJECTION} FROM original_sources WHERE id=?1");
    let record: String = connection
        .query_row(&query, [id.to_string()], |r| r.get(0))
        .optional()?
        .ok_or_else(invalid)?;
    decode(connection, &record)
}

fn decode(connection: &Connection, record: &str) -> Result<OriginalSource> {
    let source: OriginalSource = serde_json::from_str(record).map_err(|_| corrupt())?;
    validation::original(&source).map_err(|_| corrupt())?;
    let info = registered(connection, &source.digest).map_err(|_| corrupt())?;
    if info.byte_length != source.byte_length {
        return Err(corrupt());
    }
    Ok(source)
}

fn registered(connection: &Connection, digest: &AssetDigest) -> Result<AssetInfo> {
    let length: i64 = connection
        .query_row(
            "SELECT byte_length FROM assets WHERE digest=?1",
            [digest.as_str()],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(invalid)?;
    if !(0..=MAX_SOURCE_BYTES as i64).contains(&length) {
        return Err(invalid());
    }
    Ok(AssetInfo {
        digest: digest.clone(),
        byte_length: length as u64,
    })
}

pub(super) fn insert(
    connection: &Connection,
    root: &Path,
    request: &SaveSource,
) -> Result<OriginalSource> {
    let source = &request.source;
    resource(connection, source.resource_id, true)?;
    let current = list(connection, source.resource_id)?.revision;
    if current != request.expected_revision {
        return Err(AppError::new(ErrorCode::RevisionConflict));
    }
    let exists: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM original_sources WHERE id=?1)",
        [source.id.to_string()],
        |r| r.get(0),
    )?;
    if exists {
        return Err(invalid());
    }
    let info = registered(connection, &source.digest)?;
    assets::verify(root, &info).map_err(|_| corrupt())?;
    let value = original(source.clone(), current + 1, info.byte_length);
    write(connection, &value)?;
    Ok(value)
}

fn write(connection: &Connection, source: &OriginalSource) -> Result<()> {
    connection.execute("INSERT INTO original_sources(id,resource_id,digest,filename,media_type,attribution,license,origin_url,revision,byte_length) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)", params![source.id.to_string(),source.resource_id.to_string(),source.digest.as_str(),source.filename,source.media_type,source.attribution,source.license,source.origin_url,source.revision as i64,source.byte_length as i64])?;
    connection.execute("INSERT INTO source_revisions(resource_id,revision) VALUES(?1,?2) ON CONFLICT(resource_id) DO UPDATE SET revision=excluded.revision", params![source.resource_id.to_string(),source.revision as i64])?;
    Ok(())
}

pub(super) fn metadata(source: &OriginalSource) -> NewSource {
    NewSource {
        id: source.id,
        resource_id: source.resource_id,
        digest: source.digest.clone(),
        filename: source.filename.clone(),
        media_type: source.media_type.clone(),
        attribution: source.attribution.clone(),
        license: source.license.clone(),
        origin_url: source.origin_url.clone(),
    }
}

fn original(source: NewSource, revision: u64, byte_length: u64) -> OriginalSource {
    OriginalSource {
        id: source.id,
        resource_id: source.resource_id,
        digest: source.digest,
        filename: source.filename,
        media_type: source.media_type,
        attribution: source.attribution,
        license: source.license,
        origin_url: source.origin_url,
        revision,
        byte_length,
    }
}

pub(super) fn validate_native_data(connection: &Connection) -> Result<()> {
    let mut query = connection.prepare(
        "SELECT resource_id FROM original_sources UNION SELECT resource_id FROM source_revisions",
    )?;
    let mut rows = query.query([])?;
    while let Some(row) = rows.next()? {
        let id: String = row.get(0)?;
        let id = serde_json::from_value(serde_json::Value::String(id)).map_err(|_| corrupt())?;
        resource(connection, id, false)?;
        list(connection, id)?;
    }
    Ok(())
}
