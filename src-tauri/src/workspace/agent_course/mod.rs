//! Workspace-owned durable authoring groups. Desktop adapters carry typed requests.
use super::{AppError, ErrorCode, migrations::checksum};
use crate::{
    document::{BlockContent, Document},
    organization::{Command, ResourceId},
};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use uuid::Uuid;

#[cfg(target_os = "linux")]
mod delegation;
#[cfg(target_os = "linux")]
mod desktop;
#[cfg(target_os = "linux")]
mod undo;
#[cfg(target_os = "linux")]
pub(super) use delegation::Delegation;
#[cfg(target_os = "linux")]
pub(crate) use desktop::{CourseDraftRequest, CourseReviewRequest};

const MAX_DRAFTS: i64 = 64;
const MAX_GROUP_BYTES: usize = 2 * 1024 * 1024;
const MAX_CHUNKS: usize = 128;
const MAX_CHUNK_OPERATIONS: usize = 64;
const MAX_GROUP_OPERATIONS: usize = 512;

type Result<T> = std::result::Result<T, AppError>;

#[derive(Clone, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum CourseOperation {
    Organization {
        command: Command,
    },
    Document {
        resource_id: ResourceId,
        expected_revision: u64,
        document: Document,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Chunk {
    operation_id: Uuid,
    hash: String,
    operations: Vec<CourseOperation>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Status {
    Editing,
    Submitted,
    Discarded,
    Applied,
    Undone,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Record {
    version: u32,
    id: Uuid,
    base_revision: u64,
    title: String,
    chunks: Vec<Chunk>,
    status: Status,
    transition_id: Option<Uuid>,
    discard_id: Option<Uuid>,
    commit: Option<CommitReceipt>,
    undo_id: Option<Uuid>,
}

impl Record {
    fn revision(&self) -> u64 {
        1 + self.chunks.len() as u64
            + u64::from(self.transition_id.is_some())
            + u64::from(self.discard_id.is_some())
            + u64::from(self.commit.is_some())
            + u64::from(self.undo_id.is_some())
    }

    fn operation_count(&self) -> usize {
        self.chunks.iter().map(|chunk| chunk.operations.len()).sum()
    }

    fn validate(&self, id: Uuid, actor: &str) -> Result<()> {
        if self.version != 1
            || self.id != id
            || id.get_version_num() != 4
            || self.title.trim().is_empty()
            || self.title.len() > 256
            || !actor.strip_prefix("mcp:").is_some_and(valid_digest)
            || self.base_revision > super::types::MAX_REVISION as u64
            || self.chunks.len() > MAX_CHUNKS
            || self.operation_count() > MAX_GROUP_OPERATIONS
            || (matches!(
                self.status,
                Status::Submitted | Status::Applied | Status::Undone
            ) && self.transition_id.is_none())
            || (self.status == Status::Editing && self.transition_id.is_some())
            || (self.status == Status::Discarded) != self.discard_id.is_some()
            || self
                .discard_id
                .is_some_and(|id| id.get_version_num() != 4 || Some(id) == self.transition_id)
            || (matches!(
                self.status,
                Status::Submitted | Status::Applied | Status::Undone
            ) && self.chunks.is_empty())
            || (matches!(self.status, Status::Applied | Status::Undone) != self.commit.is_some())
            || (self.status == Status::Undone) != self.undo_id.is_some()
            || self.undo_id.is_some_and(|id| id.get_version_num() != 4)
            || self
                .transition_id
                .is_some_and(|id| id.get_version_num() != 4)
        {
            return Err(corrupt());
        }
        if let Some(commit) = &self.commit
            && (commit.operation_id.get_version_num() != 4
                || commit.result_revision < self.base_revision
                || commit.result_revision > super::types::MAX_REVISION as u64
                || commit.hash != proposal_hash(self)?
                || commit.documents.len() > MAX_GROUP_OPERATIONS
                || Some(commit.operation_id) == self.transition_id
                || Some(commit.operation_id) == self.undo_id
                || commit
                    .documents
                    .iter()
                    .any(|document| document.revision_id.get_version_num() != 4))
        {
            return Err(corrupt());
        }
        let mut operations = std::collections::HashSet::new();
        for chunk in &self.chunks {
            if chunk.operation_id.get_version_num() != 4
                || !operations.insert(chunk.operation_id)
                || Some(chunk.operation_id) == self.transition_id
                || Some(chunk.operation_id) == self.undo_id
                || Some(chunk.operation_id) == self.discard_id
                || self
                    .commit
                    .as_ref()
                    .is_some_and(|commit| chunk.operation_id == commit.operation_id)
                || chunk.operations.is_empty()
                || chunk.operations.len() > MAX_CHUNK_OPERATIONS
                || serde_json::to_vec(&chunk.operations)
                    .map_err(|_| corrupt())?
                    .len()
                    > 64 * 1024
                || payload_hash(&chunk.operations)? != chunk.hash
            {
                return Err(corrupt());
            }
            for operation in &chunk.operations {
                validate_operation(operation).map_err(|_| corrupt())?;
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CommitReceipt {
    operation_id: Uuid,
    hash: String,
    result_revision: u64,
    documents: Vec<DocumentReceipt>,
    delegated: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DocumentReceipt {
    resource_id: ResourceId,
    revision_id: Uuid,
}

fn proposal_hash(record: &Record) -> Result<String> {
    payload_hash(&(record.id, record.base_revision, &record.chunks))
}

fn validate_operation(operation: &CourseOperation) -> Result<()> {
    if let CourseOperation::Document {
        document,
        expected_revision,
        ..
    } = operation
        && (*expected_revision >= super::types::MAX_REVISION as u64
            || document.validate().is_err()
            || document.all_blocks().iter().any(|block| {
                matches!(
                    block.content,
                    BlockContent::Media { .. } | BlockContent::Unsupported { .. }
                )
            }))
    {
        return Err(AppError::new(ErrorCode::InvalidRequest));
    }
    Ok(())
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn payload_hash(value: &impl Serialize) -> Result<String> {
    let bytes = serde_json::to_vec(value).map_err(|_| corrupt())?;
    Ok(checksum(&bytes))
}

fn corrupt() -> AppError {
    AppError::new(ErrorCode::RecoveryRequired)
}

fn decode<T: DeserializeOwned>(data: &str) -> Result<T> {
    if data.len() > MAX_GROUP_BYTES {
        return Err(corrupt());
    }
    serde_json::from_str(data).map_err(|_| corrupt())
}

pub(super) fn validate_native_data(connection: &Connection) -> Result<()> {
    let count: i64 =
        connection.query_row("SELECT count(*) FROM agent_course_drafts", [], |row| {
            row.get(0)
        })?;
    if count > MAX_DRAFTS {
        return Err(corrupt());
    }
    let mut query =
        connection.prepare("SELECT id,actor,data FROM agent_course_drafts ORDER BY id")?;
    let mut rows = query.query([])?;
    let mut origins = std::collections::BTreeSet::new();
    while let Some(row) = rows.next()? {
        let id: String = row.get(0)?;
        let actor: String = row.get(1)?;
        let data: String = row.get(2)?;
        let id = Uuid::parse_str(&id).map_err(|_| corrupt())?;
        let record: Record = decode(&data)?;
        record.validate(id, &actor)?;
        if let Some(receipt) = &record.commit {
            for document in &receipt.documents {
                if !origins.insert((
                    document.revision_id.to_string(),
                    id.to_string(),
                    document.resource_id.to_string(),
                )) {
                    return Err(corrupt());
                }
            }
        }
    }
    let mut query = connection.prepare(
        "SELECT a.revision_id,a.proposal_id,r.resource_id
         FROM agent_authored_documents a JOIN resource_revisions r ON r.id=a.revision_id",
    )?;
    let mut rows = query.query([])?;
    while let Some(row) = rows.next()? {
        if !origins.remove(&(row.get(0)?, row.get(1)?, row.get(2)?)) {
            return Err(corrupt());
        }
    }
    if !origins.is_empty() {
        return Err(corrupt());
    }
    Ok(())
}
