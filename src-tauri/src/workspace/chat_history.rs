use super::{AppError, ErrorCode, RequestContext, database::Database, error::Result, operations};
use rusqlite::{Connection, OptionalExtension, Row, TransactionBehavior, params};
use serde::Serialize;
use uuid::Uuid;

const TITLE_BYTES: usize = 256;
const PROMPT_BYTES: usize = 32 * 1024;
const RESPONSE_BYTES: usize = 256 * 1024;
const PAGE_SIZE: i64 = 16;

/// Native service requests. Hardware, provider and authorization details stay out
/// of the course database and are never part of this request format.
pub enum ChatHistoryRequest {
    CreateThread {
        id: Uuid,
        title: String,
    },
    ListThreads {
        before: Option<i64>,
    },
    ListRuns {
        thread_id: Uuid,
        after: i64,
    },
    StartRun {
        thread_id: Uuid,
        id: Uuid,
        prompt: String,
    },
    EndRun {
        thread_id: Uuid,
        id: Uuid,
        finish: ChatRunFinish,
    },
}

pub enum ChatRunFinish {
    Completed(String),
    Cancelled,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatRunStatus {
    Streaming,
    Completed,
    Cancelled,
    Failed,
    Interrupted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatThread {
    pub id: Uuid,
    pub sequence: i64,
    pub title: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatRun {
    pub id: Uuid,
    pub thread_id: Uuid,
    pub sequence: i64,
    pub prompt: String,
    pub status: ChatRunStatus,
    pub response: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum ChatHistoryReply {
    Thread(ChatThread),
    Threads(Vec<ChatThread>),
    Run(ChatRun),
    Start { run: ChatRun, created: bool },
    Runs(Vec<ChatRun>),
}

impl ChatHistoryRequest {
    pub(super) fn admit(&self) -> Result<()> {
        let valid = match self {
            Self::CreateThread { id, title } => valid_id(*id) && valid_text(title, TITLE_BYTES),
            Self::ListThreads { before } => before.is_none_or(|sequence| sequence > 0),
            Self::ListRuns { thread_id, after } => valid_id(*thread_id) && *after >= 0,
            Self::StartRun {
                thread_id,
                id,
                prompt,
            } => valid_id(*thread_id) && valid_id(*id) && valid_text(prompt, PROMPT_BYTES),
            Self::EndRun {
                thread_id,
                id,
                finish,
            } => {
                valid_id(*thread_id)
                    && valid_id(*id)
                    && finish
                        .response()
                        .is_none_or(|text| text.len() <= RESPONSE_BYTES)
            }
        };
        if !valid {
            return Err(AppError::new(ErrorCode::InvalidRequest));
        }
        Ok(())
    }

    pub(super) fn mutates(&self) -> bool {
        matches!(
            self,
            Self::CreateThread { .. } | Self::StartRun { .. } | Self::EndRun { .. }
        )
    }
}

impl ChatRunFinish {
    fn status(&self) -> ChatRunStatus {
        match self {
            Self::Completed(_) => ChatRunStatus::Completed,
            Self::Cancelled => ChatRunStatus::Cancelled,
            Self::Failed => ChatRunStatus::Failed,
        }
    }

    fn response(&self) -> Option<&str> {
        match self {
            Self::Completed(text) => Some(text),
            Self::Cancelled | Self::Failed => None,
        }
    }
}

impl ChatRunStatus {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Streaming => "streaming",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
            Self::Failed => "failed",
            Self::Interrupted => "interrupted",
        }
    }
}

impl Database {
    pub(super) fn chat_history(
        &mut self,
        context: RequestContext,
        request: ChatHistoryRequest,
    ) -> Result<ChatHistoryReply> {
        context.authorize()?;
        if request.mutates() {
            self.authorize_workspace(context, &self.id)?;
        }
        match request {
            ChatHistoryRequest::CreateThread { id, title } => self
                .create_chat_thread(id, title)
                .map(ChatHistoryReply::Thread),
            ChatHistoryRequest::ListThreads { before } => {
                self.chat_threads(before).map(ChatHistoryReply::Threads)
            }
            ChatHistoryRequest::ListRuns { thread_id, after } => {
                self.chat_runs(thread_id, after).map(ChatHistoryReply::Runs)
            }
            ChatHistoryRequest::StartRun {
                thread_id,
                id,
                prompt,
            } => self.start_chat_run(thread_id, id, prompt),
            ChatHistoryRequest::EndRun {
                thread_id,
                id,
                finish,
            } => self
                .finish_chat_run(thread_id, id, finish)
                .map(ChatHistoryReply::Run),
        }
    }

    fn create_chat_thread(&mut self, id: Uuid, title: String) -> Result<ChatThread> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some(existing) = thread(&tx, id)? {
            return replay(existing.title == title, existing);
        }
        tx.execute(
            "INSERT INTO chat_threads (id, title) VALUES (?1, ?2)",
            params![id.to_string(), title],
        )?;
        let sequence = tx.last_insert_rowid();
        operations::durable_commit(tx)?;
        Ok(ChatThread {
            id,
            sequence,
            title,
        })
    }

    fn chat_threads(&self, before: Option<i64>) -> Result<Vec<ChatThread>> {
        let sql = match before {
            Some(_) => {
                "SELECT id, sequence, title FROM chat_threads WHERE sequence < ?1 ORDER BY sequence DESC LIMIT ?2"
            }
            None => "SELECT id, sequence, title FROM chat_threads ORDER BY sequence DESC LIMIT ?2",
        };
        let mut rows = self.connection.prepare(sql)?;
        let threads = rows.query_map(params![before, PAGE_SIZE], decode_thread)?;
        threads
            .collect::<rusqlite::Result<_>>()
            .map_err(AppError::from)
    }

    fn chat_runs(&self, thread_id: Uuid, after: i64) -> Result<Vec<ChatRun>> {
        if thread(&self.connection, thread_id)?.is_none() {
            return Err(AppError::new(ErrorCode::InvalidReference));
        }
        let mut rows = self.connection.prepare(
            "SELECT id, thread_id, sequence, prompt, status, response FROM chat_runs
             WHERE thread_id = ?1 AND sequence > ?2 ORDER BY sequence LIMIT ?3",
        )?;
        let runs = rows.query_map(params![thread_id.to_string(), after, PAGE_SIZE], decode_run)?;
        runs.collect::<rusqlite::Result<_>>()
            .map_err(AppError::from)
    }

    fn start_chat_run(
        &mut self,
        thread_id: Uuid,
        id: Uuid,
        prompt: String,
    ) -> Result<ChatHistoryReply> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some(existing) = run(&tx, id)? {
            return replay(
                existing.thread_id == thread_id && existing.prompt == prompt,
                ChatHistoryReply::Start {
                    run: existing,
                    created: false,
                },
            );
        }
        if thread(&tx, thread_id)?.is_none() {
            return Err(AppError::new(ErrorCode::InvalidReference));
        }
        let active: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM chat_runs WHERE thread_id = ?1 AND status = 'streaming')",
            [thread_id.to_string()],
            |row| row.get(0),
        )?;
        if active {
            return Err(AppError::new(ErrorCode::Busy));
        }
        tx.execute(
            "INSERT INTO chat_runs (id, thread_id, prompt, status) VALUES (?1, ?2, ?3, 'streaming')",
            params![id.to_string(), thread_id.to_string(), prompt],
        )?;
        let sequence = tx.last_insert_rowid();
        operations::durable_commit(tx)?;
        Ok(ChatHistoryReply::Start {
            run: ChatRun {
                id,
                thread_id,
                sequence,
                prompt,
                status: ChatRunStatus::Streaming,
                response: None,
            },
            created: true,
        })
    }

    fn finish_chat_run(
        &mut self,
        thread_id: Uuid,
        id: Uuid,
        finish: ChatRunFinish,
    ) -> Result<ChatRun> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut existing = run(&tx, id)?
            .filter(|run| run.thread_id == thread_id)
            .ok_or_else(|| AppError::new(ErrorCode::InvalidReference))?;
        let status = finish.status();
        let response = finish.response();
        if existing.status != ChatRunStatus::Streaming {
            return replay(
                existing.status == status && existing.response.as_deref() == response,
                existing,
            );
        }
        tx.execute(
            "UPDATE chat_runs SET status = ?2, response = ?3 WHERE id = ?1",
            params![id.to_string(), status.as_str(), response],
        )?;
        operations::durable_commit(tx)?;
        existing.status = status;
        existing.response = response.map(str::to_owned);
        Ok(existing)
    }

    pub(super) fn interrupt_chat_runs(&self) -> Result<()> {
        if !self.recovery.blocks_writes() {
            self.connection.execute(
                "UPDATE chat_runs SET status = 'interrupted' WHERE status = 'streaming'",
                [],
            )?;
        }
        Ok(())
    }
}

fn decode_thread(row: &Row<'_>) -> rusqlite::Result<ChatThread> {
    Ok(ChatThread {
        id: decode_id(row, 0)?,
        sequence: row.get(1)?,
        title: row.get(2)?,
    })
}

fn decode_run(row: &Row<'_>) -> rusqlite::Result<ChatRun> {
    let status: String = row.get(4)?;
    let status = match status.as_str() {
        "streaming" => ChatRunStatus::Streaming,
        "completed" => ChatRunStatus::Completed,
        "cancelled" => ChatRunStatus::Cancelled,
        "failed" => ChatRunStatus::Failed,
        "interrupted" => ChatRunStatus::Interrupted,
        _ => return Err(rusqlite::Error::InvalidQuery),
    };
    Ok(ChatRun {
        id: decode_id(row, 0)?,
        thread_id: decode_id(row, 1)?,
        sequence: row.get(2)?,
        prompt: row.get(3)?,
        status,
        response: row.get(5)?,
    })
}

fn thread(connection: &Connection, id: Uuid) -> Result<Option<ChatThread>> {
    Ok(connection
        .query_row(
            "SELECT id, sequence, title FROM chat_threads WHERE id = ?1",
            [id.to_string()],
            decode_thread,
        )
        .optional()?)
}

fn run(connection: &Connection, id: Uuid) -> Result<Option<ChatRun>> {
    Ok(connection
        .query_row(
            "SELECT id, thread_id, sequence, prompt, status, response FROM chat_runs WHERE id = ?1",
            [id.to_string()],
            decode_run,
        )
        .optional()?)
}

fn decode_id(row: &Row<'_>, index: usize) -> rusqlite::Result<Uuid> {
    let raw: String = row.get(index)?;
    Uuid::parse_str(&raw)
        .ok()
        .filter(|id| valid_id(*id) && id.to_string() == raw)
        .ok_or(rusqlite::Error::InvalidQuery)
}

fn valid_id(id: Uuid) -> bool {
    id.get_version_num() == 4
}

fn valid_text(value: &str, max_bytes: usize) -> bool {
    value.len() <= max_bytes && !value.trim().is_empty()
}

fn replay<T>(same_input: bool, value: T) -> Result<T> {
    if !same_input {
        return Err(AppError::new(ErrorCode::IdempotencyConflict));
    }
    Ok(value)
}

pub(super) fn validate_native_data(connection: &Connection) -> Result<()> {
    let invalid = |_| AppError::new(ErrorCode::RecoveryRequired);
    let mut threads = connection.prepare("SELECT id, sequence, title FROM chat_threads")?;
    for row in threads.query_map([], decode_thread)? {
        if !valid_text(&row.map_err(invalid)?.title, TITLE_BYTES) {
            return Err(AppError::new(ErrorCode::RecoveryRequired));
        }
    }
    let mut runs = connection
        .prepare("SELECT id, thread_id, sequence, prompt, status, response FROM chat_runs")?;
    for row in runs.query_map([], decode_run)? {
        if !valid_text(&row.map_err(invalid)?.prompt, PROMPT_BYTES) {
            return Err(AppError::new(ErrorCode::RecoveryRequired));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
