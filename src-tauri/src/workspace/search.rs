//! Bounded live literal discovery. No index, query language or source ingestion.
use super::{database::Database, error::Result, *};
use crate::organization::ResourceId;
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};

const PAGE_SIZE: usize = 20;
const SCAN_SIZE: usize = 200;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SearchHit {
    pub resource_id: ResourceId,
    pub title: String,
    pub snippet: String,
    pub references: Vec<SearchReference>,
    pub references_truncated: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SearchReference {
    pub source_id: uuid::Uuid,
    pub locator: ExtractionLocator,
    pub original_available: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SearchPage {
    pub revision: u64,
    pub items: Vec<SearchHit>,
    pub next_offset: Option<u64>,
    pub index_rebuilt: bool,
}

impl Database {
    pub(super) fn reset_search_index(&self, context: RequestContext) -> Result<()> {
        context.authorize()?;
        let tx = self.connection.unchecked_transaction()?;
        tx.execute_batch("DROP TABLE IF EXISTS search_fts; CREATE VIRTUAL TABLE search_fts USING fts5(resource_id UNINDEXED, title, body, tags, tokenize='trigram'); UPDATE search_index_state SET dirty=1 WHERE id=1;")?;
        tx.commit()?;
        self.rebuild_search_index(context)
    }
    pub(super) fn rebuild_search_index(&self, context: RequestContext) -> Result<()> {
        context.authorize()?;
        let tx = self.connection.unchecked_transaction()?;
        tx.execute("DELETE FROM search_fts", [])?;
        tx.execute("DELETE FROM search_projection", [])?;
        {
            let mut resources =
                tx.prepare("SELECT id,title FROM resources WHERE archived=0 ORDER BY id")?;
            let rows = resources
                .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
            for row in rows {
                let (raw_id, title) = row?;
                stored_title(&title)?;
                let id: ResourceId =
                    serde_json::from_value(serde_json::Value::String(raw_id.clone()))
                        .map_err(|_| corrupt())?;
                let body = self
                    .read_document(context, id, None)?
                    .map(|x| x.document.text())
                    .unwrap_or_default();
                let mut tags = String::new();
                let mut statement = tx.prepare("SELECT t.title FROM tags t JOIN tag_resources tr ON tr.tag_id=t.id WHERE tr.resource_id=?1 ORDER BY t.id")?;
                for tag in statement.query_map([&raw_id], |r| r.get::<_, String>(0))? {
                    let tag = tag?;
                    stored_title(&tag)?;
                    tags.push_str(&tag);
                    tags.push('\n');
                }
                tx.execute(
                    "INSERT INTO search_fts(resource_id,title,body,tags) VALUES(?1,?2,?3,?4)",
                    params![
                        raw_id,
                        title.to_lowercase(),
                        body.to_lowercase(),
                        tags.to_lowercase()
                    ],
                )?;
                tx.execute("INSERT INTO search_projection(resource_id,title,body,tags) SELECT resource_id,title,body,tags FROM search_fts WHERE rowid=last_insert_rowid()", [])?;
            }
        }
        tx.execute("UPDATE search_index_state SET dirty=0 WHERE id=1", [])?;
        tx.commit()?;
        Ok(())
    }
    pub(super) fn search_resources(
        &self,
        context: RequestContext,
        query: &str,
        offset: u64,
    ) -> Result<SearchPage> {
        context.authorize()?;
        validate_query(query, offset)?;
        let (dirty, indexed, resources): (bool, i64, i64) = self.connection.query_row("SELECT dirty,(SELECT count(*) FROM search_projection),(SELECT count(*) FROM resources WHERE archived=0) FROM search_index_state WHERE id=1", [], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?;
        let integrity = self.connection.execute(
            "INSERT INTO search_fts(search_fts) VALUES('integrity-check')",
            [],
        );
        let mut reconstructed = false;
        if integrity.is_err() {
            self.reset_search_index(context)?;
            reconstructed = true;
        }
        let mismatch: bool = self.connection.query_row("SELECT (SELECT count(*) FROM search_fts)<>(SELECT count(*) FROM search_projection) OR EXISTS(SELECT resource_id,title,body,tags FROM search_fts EXCEPT SELECT resource_id,title,body,tags FROM search_projection) OR EXISTS(SELECT resource_id,title,body,tags FROM search_projection EXCEPT SELECT resource_id,title,body,tags FROM search_fts)", [], |r| r.get(0))?;
        let index_rebuilt = dirty || indexed != resources || reconstructed || mismatch;
        if index_rebuilt && !reconstructed {
            self.rebuild_search_index(context)?;
        }
        let revision = self.search_revision()?;
        let needle = query.to_lowercase();
        let rows = self.search_rows(offset, &needle)?;
        let mut items = Vec::new();
        let mut availability = std::collections::HashMap::new();
        let mut consumed = 0;
        for (id, title, candidate) in rows.iter().take(SCAN_SIZE) {
            consumed += 1;
            if !candidate {
                continue;
            }
            if let Some(hit) =
                self.match_resource(context, id, title, &needle, &mut availability)?
            {
                items.push(hit);
                if items.len() == PAGE_SIZE {
                    break;
                }
            }
        }
        Ok(SearchPage {
            revision,
            items,
            next_offset: continuation(offset, consumed, rows.len()),
            index_rebuilt,
        })
    }

    fn search_revision(&self) -> Result<u64> {
        let revision: i64 = self
            .connection
            .query_row(
                "SELECT revision FROM library_meta WHERE workspace_id=?1",
                [self.id.as_str()],
                |r| r.get(0),
            )
            .optional()?
            .unwrap_or(0);
        if !(0..=super::types::MAX_REVISION).contains(&revision) {
            return Err(corrupt());
        }
        Ok(revision as u64)
    }

    fn search_rows(&self, offset: u64, needle: &str) -> Result<Vec<(String, String, bool)>> {
        if needle.chars().count() >= 3 {
            let phrase = format!("\"{}\"", needle.replace('"', "\"\""));
            return Ok(self.connection.prepare("SELECT id,title,id IN (SELECT resource_id FROM search_fts WHERE search_fts MATCH ?3) FROM resources WHERE archived=0 ORDER BY id LIMIT ?1 OFFSET ?2")?
                .query_map(params![(SCAN_SIZE + 1) as i64, offset as i64, phrase], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
                .collect::<rusqlite::Result<Vec<_>>>()?);
        }
        Ok(self
            .connection
            .prepare(
                "SELECT id,title FROM resources WHERE archived=0 ORDER BY id LIMIT ?1 OFFSET ?2",
            )?
            .query_map(params![(SCAN_SIZE + 1) as i64, offset as i64], |r| {
                Ok((r.get(0)?, r.get(1)?, true))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?)
    }

    fn match_resource(
        &self,
        context: RequestContext,
        raw_id: &str,
        title: &str,
        needle: &str,
        availability: &mut std::collections::HashMap<String, bool>,
    ) -> Result<Option<SearchHit>> {
        stored_title(title)?;
        let resource_id: ResourceId =
            serde_json::from_value(serde_json::Value::String(raw_id.into()))
                .map_err(|_| corrupt())?;
        // Reuse authoritative revision/digest/document validation, including
        // resource ownership. Search never inspects serialized schema keys.
        let body = self
            .read_document(context, resource_id, None)?
            .map(|saved| saved.document.text())
            .unwrap_or_default();
        if !title.to_lowercase().contains(needle)
            && !body.to_lowercase().contains(needle)
            && !self.matches_tag(raw_id, needle)?
        {
            return Ok(None);
        }
        let (references, references_truncated) =
            self.search_references(context, resource_id, availability)?;
        Ok(Some(SearchHit {
            resource_id,
            title: title.into(),
            snippet: body.chars().take(160).collect(),
            references,
            references_truncated,
        }))
    }

    fn search_references(
        &self,
        context: RequestContext,
        resource_id: ResourceId,
        availability: &mut std::collections::HashMap<String, bool>,
    ) -> Result<(Vec<SearchReference>, bool)> {
        let mut query = self.connection.prepare("SELECT j.id,CAST(json_extract(o.value,'$.index') AS INTEGER) FROM ingestion_jobs j,json_each(j.data,'$.outcomes') o JOIN resource_heads h ON h.resource_id=json_extract(o.value,'$.resourceId') AND h.revision_id=json_extract(o.value,'$.documentRevisionId') WHERE h.resource_id=?1 AND json_extract(j.data,'$.status')<>'undone' ORDER BY j.rowid LIMIT 21")?;
        let rows = query
            .query_map([resource_id.to_string()], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let mut result = Vec::new();
        for (id, index) in rows.iter().take(20) {
            let id = uuid::Uuid::parse_str(id).map_err(|_| corrupt())?;
            let index = usize::try_from(*index).map_err(|_| corrupt())?;
            let job = self.ingestion_job(context, id)?;
            let source = self
                .sources(context, job.source.resource_id)?
                .sources
                .into_iter()
                .find(|source| source.id == job.source.id)
                .ok_or_else(corrupt)?;
            if source != job.source {
                return Err(corrupt());
            }
            let available = *availability
                .entry(source.digest.as_str().to_owned())
                .or_insert_with(|| {
                    assets::verify(
                        self.owner.root(),
                        &AssetInfo {
                            digest: source.digest.clone(),
                            byte_length: source.byte_length,
                        },
                    )
                    .is_ok()
                });
            let locator = job
                .extraction
                .records
                .get(index)
                .ok_or_else(corrupt)?
                .locator
                .clone();
            result.push(SearchReference {
                source_id: job.source.id,
                locator,
                original_available: available,
            });
        }
        Ok((result, rows.len() > 20))
    }

    fn matches_tag(&self, resource_id: &str, needle: &str) -> Result<bool> {
        let mut statement = self.connection.prepare("SELECT t.title FROM tags t JOIN tag_resources tr ON tr.tag_id=t.id WHERE tr.resource_id=?1 ORDER BY t.id")?;
        let titles = statement.query_map([resource_id], |r| r.get::<_, String>(0))?;
        for title in titles {
            let title = title?;
            stored_title(&title)?;
            if title.to_lowercase().contains(needle) {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

fn continuation(offset: u64, consumed: usize, available: usize) -> Option<u64> {
    if consumed < available {
        Some(offset + consumed as u64)
    } else {
        None
    }
}

fn validate_query(query: &str, offset: u64) -> Result<()> {
    if query.chars().count() > 256
        || query.contains('\0')
        || offset > super::types::MAX_REVISION as u64
    {
        return Err(invalid());
    }
    Ok(())
}
fn stored_title(title: &str) -> Result<()> {
    if title.trim().is_empty() || title.chars().count() > 256 || title.contains('\0') {
        return Err(corrupt());
    }
    Ok(())
}
fn invalid() -> AppError {
    AppError::new(ErrorCode::InvalidRequest)
}
fn corrupt() -> AppError {
    AppError::new(ErrorCode::RecoveryRequired)
}

#[cfg(test)]
mod tests;
