//! Course-local resume discovery over durable sessions, without a second ledger.
use super::*;
use crate::workspace::{RequestContext, database::Database};
use serde::Serialize;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResumeSession {
    pub session_id: Uuid,
    pub unit_id: String,
    pub title: String,
    pub composition_revision_id: String,
    pub cursor: usize,
    pub total: usize,
    pub mode: StudyMode,
    pub phase: StudyPhase,
    pub saved_response: bool,
    pub changed_content: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResumePage {
    pub contract_version: u32,
    pub offset: u64,
    pub next_offset: Option<u64>,
    pub sessions: Vec<ResumeSession>,
}
impl Database {
    pub(in crate::workspace) fn study_resume_sessions(
        &self,
        context: RequestContext,
        offset: u64,
    ) -> Result<ResumePage> {
        context.authorize()?;
        // Operation row order survives native and opted-in portable history
        // recovery. Wall time may be absent or roll backwards; do not rank by it.
        // Find the last operation only for eligible sessions. Grouping the whole
        // ledger would also visit operations for every completed session.
        let rows = self.connection.prepare(
            "SELECT s.id,s.unit_id,u.title FROM study_sessions s JOIN units u ON u.id=s.unit_id
             WHERE u.archived=0 AND json_extract(s.data,'$.cursor') < json_array_length(s.data,'$.items')
             ORDER BY (SELECT max(rowid) FROM study_operations WHERE session_id=s.id) DESC,
                      s.rowid DESC LIMIT 65 OFFSET ?1",
        )?.query_map([i64::try_from(offset).map_err(|_| invalid())?], |r| {
            Ok((r.get::<_, String>(0)?,r.get::<_, String>(1)?,r.get::<_, String>(2)?))
        })?.collect::<rusqlite::Result<Vec<_>>>()?;
        let next_offset = (rows.len() > 64).then_some(offset + 64);
        let mut sessions = Vec::new();
        for (id, unit_id, title) in rows.into_iter().take(64) {
            // Reuse canonical session validation without loading or exposing
            // judgments/answers through this small navigation projection.
            let s = storage::load(
                &self.connection,
                Uuid::parse_str(&id).map_err(|_| corrupt())?,
            )?;
            let current = self.lesson_read(context, &unit_id, None)?;
            let changed_content = current
                .as_ref()
                .is_none_or(|v| v.id != s.composition_revision_id);
            if s.unit_id != unit_id || s.cursor >= s.items.len() {
                return Err(corrupt());
            }
            sessions.push(ResumeSession {
                session_id: s.id,
                unit_id,
                title,
                composition_revision_id: s.composition_revision_id,
                cursor: s.cursor,
                total: s.items.len(),
                mode: s.mode,
                phase: s.phase,
                saved_response: s.draft.response.is_some(),
                changed_content,
            });
        }
        Ok(ResumePage {
            contract_version: 1,
            offset,
            next_offset,
            sessions,
        })
    }
}
