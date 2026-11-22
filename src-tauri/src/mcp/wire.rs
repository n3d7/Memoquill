use crate::workspace::{
    agent_course::CourseDraftRequest, mcp_drafts::DraftRequest, mcp_reads::CourseRead,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::{env, fs, io, os::unix::fs::MetadataExt, path::PathBuf, time::Duration};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub(super) const REQUEST_LIMIT: usize = 64 * 1024;
pub(super) const REPLY_LIMIT: usize = 1024 * 1024;
pub(super) const IO_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum Request {
    Status,
    Read { request: CourseRead },
    Draft { request: DraftRequest },
    CourseDraft { request: CourseDraftRequest },
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum Reply {
    Success { result: Value },
    Error { code: String, message: String },
}

pub(super) fn socket_path() -> io::Result<PathBuf> {
    let root = env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .filter(|root| root.is_absolute())
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Private XDG runtime required"))?;
    verify_dir(&root)?;
    Ok(root.join("memoquill-mcp").join("broker.sock"))
}

pub(super) fn uid() -> u32 {
    rustix::process::geteuid().as_raw()
}

pub(super) fn verify_dir(path: &std::path::Path) -> io::Result<()> {
    let meta = fs::symlink_metadata(path)?;
    if !meta.is_dir() || meta.uid() != uid() || meta.mode() & 0o077 != 0 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Unsafe runtime directory",
        ));
    }
    Ok(())
}

pub(super) async fn receive<T: DeserializeOwned>(
    stream: &mut (impl AsyncRead + Unpin),
    limit: usize,
) -> io::Result<T> {
    let len = stream.read_u32().await? as usize;
    if len == 0 || len > limit {
        return Err(io::ErrorKind::InvalidData.into());
    }
    let mut data = vec![0; len];
    stream.read_exact(&mut data).await?;
    serde_json::from_slice(&data).map_err(|_| io::ErrorKind::InvalidData.into())
}

pub(super) async fn send(stream: &mut (impl AsyncWrite + Unpin), bytes: &[u8]) -> io::Result<()> {
    stream.write_u32(bytes.len() as u32).await?;
    stream.write_all(bytes).await
}
