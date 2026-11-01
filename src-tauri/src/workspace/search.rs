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
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SearchPage {
    pub revision: u64,
    pub items: Vec<SearchHit>,
    pub next_offset: Option<u64>,
}

impl Database {
    pub(super) fn search_resources(
        &self,
        context: RequestContext,
        query: &str,
        offset: u64,
    ) -> Result<SearchPage> {
        context.authorize()?;
        validate_query(query, offset)?;
        let revision = self.search_revision()?;
        let rows = self.search_rows(offset)?;
        let needle = query.to_lowercase();
        let mut items = Vec::new();
        let mut consumed = 0;
        for (id, title) in rows.iter().take(SCAN_SIZE) {
            consumed += 1;
            if let Some(hit) = self.match_resource(context, id, title, &needle)? {
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

    fn search_rows(&self, offset: u64) -> Result<Vec<(String, String)>> {
        Ok(self
            .connection
            .prepare(
                "SELECT id,title FROM resources WHERE archived=0 ORDER BY id LIMIT ?1 OFFSET ?2",
            )?
            .query_map(params![(SCAN_SIZE + 1) as i64, offset as i64], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?)
    }

    fn match_resource(
        &self,
        context: RequestContext,
        raw_id: &str,
        title: &str,
        needle: &str,
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
        Ok(Some(SearchHit {
            resource_id,
            title: title.into(),
            snippet: body.chars().take(160).collect(),
        }))
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
