use super::*;
use rusqlite::{limits::Limit, types::ValueRef};
use std::collections::{BTreeMap, BTreeSet};

type Schema = Vec<(String, String, String, Option<String>)>;
fn schema(connection: &Connection) -> Result<Schema> {
    let mut query = connection
        .prepare("SELECT type,name,tbl_name,sql FROM sqlite_schema ORDER BY type,name LIMIT 129")?;
    let rows = query.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?;
    let result: Schema = rows.collect::<std::result::Result<_, _>>()?;
    if result.len() > 128 {
        return Err(budget());
    }
    Ok(result)
}

pub(super) fn asset_rows(connection: &Connection) -> Result<Vec<(AssetDigest, u64)>> {
    let mut query =
        connection.prepare("SELECT digest,byte_length FROM assets ORDER BY digest LIMIT 4097")?;
    let values = query
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    if values.len() > MAX_ENTRIES - 2 {
        return Err(budget());
    }
    values
        .into_iter()
        .map(|(digest, size)| {
            if size < 0 || size as u64 > MAX_ASSET {
                return Err(budget());
            }
            Ok((
                AssetDigest::try_from(digest).map_err(|_| invalid())?,
                size as u64,
            ))
        })
        .collect()
}

pub(super) fn table_counts(connection: &Connection) -> Result<BTreeMap<String, u64>> {
    let mut result = BTreeMap::new();
    for (kind, name, _, _) in schema(connection)? {
        if kind == "table" {
            let quoted = name.replace('"', "\"\"");
            let count: i64 =
                connection.query_row(&format!("SELECT count(*) FROM \"{quoted}\""), [], |r| {
                    r.get(0)
                })?;
            if count > 1_000_000 {
                return Err(budget());
            }
            result.insert(name, count as u64);
        }
    }
    Ok(result)
}

pub(super) fn validate(path: &Path, manifest: &NativeManifest) -> Result<()> {
    if std::fs::symlink_metadata(path)?.len() > MAX_DATABASE {
        return Err(budget());
    }
    let connection = database::open_connection(path, database::Access::ReadOnly)?;
    connection.set_limit(Limit::SQLITE_LIMIT_LENGTH, 8 * 1024 * 1024)?;
    connection.set_limit(Limit::SQLITE_LIMIT_SQL_LENGTH, 65536)?;
    let app: i32 = connection.pragma_query_value(None, "application_id", |r| r.get(0))?;
    let version: i64 = connection.pragma_query_value(None, "user_version", |r| r.get(0))?;
    if app != database::APPLICATION_ID
        || version != SCHEMA_VERSION
        || version != manifest.schema_version
    {
        return Err(AppError::new(ErrorCode::UnsupportedSchema));
    }
    // The only DDL executed comes from our bundled migrations in a DIFFERENT database.
    // Compare exact object definitions before touching any untrusted domain objects.
    let trusted = tempfile::Builder::new()
        .permissions(std::fs::Permissions::from_mode(0o700))
        .tempdir()?;
    let expected_path = trusted.path().join("expected.sqlite3");
    database::bootstrap(&expected_path, SCHEMA_VERSION)?;
    let expected = database::open_connection(&expected_path, database::Access::ReadOnly)?;
    let objects = schema(&connection)?;
    if objects != schema(&expected)? {
        return Err(invalid());
    }
    super::super::migrations::validate(&connection, SCHEMA_VERSION)?;
    database::integrity(&connection)?;
    let mut total_rows = 0usize;
    let mut actual_counts = BTreeMap::new();
    for (kind, name, _, _) in &objects {
        if kind != "table" {
            continue;
        }
        // Names are now from the trusted exact schema, still quote identifiers.
        let quoted = name.replace('"', "\"\"");
        let mut query = connection.prepare(&format!("SELECT * FROM \"{quoted}\" LIMIT 1000001"))?;
        let columns = query.column_count();
        let mut rows = query.query([])?;
        let mut count = 0;
        while let Some(row) = rows.next()? {
            count += 1;
            total_rows += 1;
            if count > 1_000_000 || total_rows > 2_000_000 {
                return Err(budget());
            }
            for index in 0..columns {
                match row.get_ref(index)? {
                    ValueRef::Text(bytes) | ValueRef::Blob(bytes)
                        if bytes.len() > 8 * 1024 * 1024 =>
                    {
                        return Err(budget());
                    }
                    _ => {}
                }
            }
        }
        actual_counts.insert(name.clone(), count as u64);
    }
    if actual_counts != manifest.table_counts {
        return Err(invalid());
    }
    let count: i64 =
        connection.query_row("SELECT count(*) FROM workspace_meta", [], |r| r.get(0))?;
    let id: String =
        connection.query_row("SELECT id FROM workspace_meta WHERE singleton=1", [], |r| {
            r.get(0)
        })?;
    if count != 1 || id != manifest.workspace_id.as_str() {
        return Err(invalid());
    }
    let (preference_count,revision,data):(i64,i64,String)=connection.query_row("SELECT (SELECT count(*) FROM preferences),revision,data FROM preferences WHERE workspace_id=?1",[&id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?;
    if preference_count != 1
        || !(0..=super::super::types::MAX_REVISION).contains(&revision)
        || serde_json::from_str::<ReadingPreferences>(&data).is_err()
    {
        return Err(invalid());
    }
    super::super::content::validate_native_data(&connection, &manifest.workspace_id)?;
    super::super::lessons::validate_native_data(&connection)?;
    let mut jobs = connection.prepare("SELECT id,source_id,data FROM ingestion_jobs")?;
    let mut rows = jobs.query([])?;
    while let Some(row) = rows.next()? {
        let id: String = row.get(0)?;
        let source: String = row.get(1)?;
        let data: String = row.get(2)?;
        let job: IngestionJob = serde_json::from_str(&data).map_err(|_| invalid())?;
        if job.id.to_string() != id
            || job.source.id.to_string() != source
            || !matches!(job.handler_version, 1 | 2)
        {
            return Err(invalid());
        }
    }
    for sql in [
        "SELECT count(*) FROM resource_heads h JOIN resource_revisions r ON r.id=h.revision_id WHERE h.resource_id!=r.resource_id",
        "SELECT count(*) FROM annotations a JOIN resource_revisions r ON r.id=a.revision_id WHERE a.resource_id!=r.resource_id",
        "SELECT count(*) FROM reading_positions p JOIN resource_revisions r ON r.id=p.revision_id WHERE p.resource_id!=r.resource_id",
        "SELECT count(*) FROM lesson_heads h JOIN lesson_revisions r ON r.id=h.revision_id WHERE h.unit_id!=r.unit_id",
        "SELECT count(*) FROM lesson_activity_heads h JOIN lesson_activity_revisions r ON r.id=h.revision_id WHERE h.unit_id!=r.unit_id OR h.activity_id!=r.activity_id",
        "SELECT count(*) FROM original_sources s JOIN assets a ON a.digest=s.digest WHERE s.byte_length!=a.byte_length",
        "SELECT count(*) FROM lesson_dependencies d WHERE (d.kind='resource' AND NOT EXISTS(SELECT 1 FROM resource_revisions r WHERE r.resource_id=d.target_id AND r.id=d.revision_id)) OR (d.kind='source' AND NOT EXISTS(SELECT 1 FROM lesson_source_bindings b WHERE b.source_id=d.target_id AND b.id=d.revision_id)) OR (d.kind='activity' AND NOT EXISTS(SELECT 1 FROM lesson_activity_revisions a WHERE a.activity_id=d.target_id AND a.id=d.revision_id))",
    ] {
        if connection.query_row(sql, [], |r| r.get::<_, i64>(0))? != 0 {
            return Err(invalid());
        }
    }
    let registered = asset_rows(&connection)?;
    let declared: BTreeMap<_, _> = manifest
        .payloads
        .iter()
        .filter(|p| p.name.starts_with("assets/"))
        .map(|p| (p.sha256.clone(), p.byte_length))
        .collect();
    let missing: BTreeSet<_> = manifest
        .missing_assets
        .iter()
        .map(|d| d.as_str().to_string())
        .collect();
    if missing.len() != manifest.missing_assets.len()
        || declared.len() + missing.len() != registered.len()
    {
        return Err(invalid());
    }
    for (digest, length) in registered {
        match declared.get(digest.as_str()) {
            Some(size) if *size == length && !missing.contains(digest.as_str()) => {}
            None if missing.contains(digest.as_str()) => {}
            _ => return Err(invalid()),
        }
    }
    Ok(())
}
