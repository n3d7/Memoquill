use super::*;
pub(super) fn source_snapshot(
    c: &Connection,
    unit_id: &str,
    source_id: &str,
    locator: Option<String>,
    declared_used_ranges: Vec<String>,
) -> Result<SourceBindingRevision> {
    uuid(source_id)?;
    if declared_used_ranges.len() > 64
        || declared_used_ranges
            .iter()
            .any(|s| s.trim().is_empty() || s.len() > 1024 || s.contains('\0'))
        || locator
            .as_ref()
            .is_some_and(|s| s.trim().is_empty() || s.len() > 1024 || s.contains('\0'))
    {
        return Err(invalid());
    }
    type SourceMetadata = (
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
    );
    let row:Option<SourceMetadata>=c.query_row("SELECT digest,filename,attribution,license,origin_url FROM original_sources WHERE id=?1",[source_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).optional()?;
    let (digest, filename, attribution, license, origin_url) = row.ok_or_else(invalid)?;
    Ok(SourceBindingRevision {
        id: Uuid::new_v4().to_string(),
        unit_id: unit_id.into(),
        source_id: source_id.into(),
        asset_digest: digest,
        filename,
        attribution,
        license,
        origin_url,
        locator,
        declared_used_ranges,
        extraction_coverage: "notExtracted".into(),
        read_coverage: "unknown".into(),
        warnings: vec![
            "Original source only; declared ranges do not establish extraction or reading.".into(),
        ],
    })
}
pub(super) fn validate_source(c: &Connection, s: &SourceBindingRevision, unit: &str) -> Result<()> {
    if s.unit_id != unit
        || s.extraction_coverage != "notExtracted"
        || s.read_coverage != "unknown"
        || bytes(s)?.len() > 131072
    {
        return Err(invalid());
    }
    uuid(&s.id)?;
    let expected = source_snapshot(
        c,
        unit,
        &s.source_id,
        s.locator.clone(),
        s.declared_used_ranges.clone(),
    )?;
    if s.asset_digest != expected.asset_digest
        || s.filename != expected.filename
        || s.attribution != expected.attribution
        || s.license != expected.license
        || s.origin_url != expected.origin_url
        || s.warnings != expected.warnings
    {
        return Err(invalid());
    }
    let exists: bool = c.query_row(
        "SELECT EXISTS(SELECT 1 FROM assets WHERE digest=?1)",
        [&s.asset_digest],
        |r| r.get(0),
    )?;
    if !exists {
        return Err(invalid());
    }
    Ok(())
}
impl Database {
    pub(in crate::workspace) fn lesson_source(
        &self,
        context: RequestContext,
        unit_id: &str,
        revision: &str,
    ) -> Result<Option<SourceBindingRevision>> {
        context.authorize()?;
        unit(&self.connection, unit_id, false)?;
        uuid(revision)?;
        for draft in self.lesson_list_drafts(context, unit_id)? {
            let s = state(&self.connection, &draft.id, unit_id)?;
            for dependency in s.dependencies {
                if let Pending::Source { snapshot } = dependency
                    && snapshot.id == revision
                {
                    return Ok(Some(snapshot));
                }
            }
        }
        let row: Option<(String, String)> = self
            .connection
            .query_row(
                "SELECT data,digest FROM lesson_source_bindings WHERE id=?1 AND unit_id=?2",
                params![revision, unit_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        row.map(|(data, digest)| {
            if data.len() > 131072 {
                return Err(corrupt());
            }
            let snapshot: SourceBindingRevision =
                serde_json::from_str(&data).map_err(|_| corrupt())?;
            validate_source(&self.connection, &snapshot, unit_id).map_err(|_| corrupt())?;
            if snapshot.id != revision || hash(&snapshot)? != digest {
                return Err(corrupt());
            }
            Ok(snapshot)
        })
        .transpose()
    }
}
