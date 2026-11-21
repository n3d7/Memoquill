use super::wire::{self, IO_TIMEOUT, REPLY_LIMIT, REQUEST_LIMIT, Reply, Request};
use crate::workspace::{
    AppError, CourseManager, ErrorCode,
    mcp_reads::{CourseReads, ReadPermissions},
};
use serde::Serialize;
use serde_json::json;
use std::{
    collections::{HashMap, VecDeque},
    fs, io,
    os::unix::fs::{DirBuilderExt, FileTypeExt, MetadataExt, PermissionsExt},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::{
    net::{UnixListener, UnixStream},
    sync::{oneshot, watch},
    time::timeout,
};
use uuid::Uuid;

const MAX_CONNECTIONS: usize = 8;
const PAIR_WINDOW: Duration = Duration::from_secs(300);
const GRANT_WINDOW: Duration = Duration::from_secs(3600);

#[derive(Default, Clone)]
pub(crate) struct Connections(Arc<Mutex<State>>);

#[derive(Default)]
struct State {
    broker: Option<watch::Sender<bool>>,
    closed: Option<oneshot::Receiver<()>>,
    peers: HashMap<Uuid, Peer>,
    audit: VecDeque<AuditEntry>,
}

struct Peer {
    pid: Option<i32>,
    expires: Instant,
    grant: Option<Grant>,
    _revoked: watch::Sender<bool>,
}

struct Grant {
    course_id: Uuid,
    permissions: ReadPermissions,
    reads: CourseReads,
}

impl Drop for Grant {
    fn drop(&mut self) {
        self.reads.revoke_delegation();
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Status {
    enabled: bool,
    peers: Vec<PeerStatus>,
    recent_events: Vec<AuditEntry>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PeerStatus {
    challenge: Uuid,
    pid: Option<i32>,
    course_id: Option<Uuid>,
    permissions: Option<ReadPermissions>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AuditEntry {
    action: AuditAction,
    at_unix_ms: u64,
    course_id: Option<Uuid>,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum AuditAction {
    Enabled,
    Disabled,
    Approved,
    Revoked,
    Disconnected,
    Expired,
}

impl State {
    fn record(&mut self, action: AuditAction, course_id: Option<Uuid>) {
        if self.audit.len() >= 32 {
            self.audit.pop_front();
        }
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        self.audit.push_back(AuditEntry {
            action,
            at_unix_ms: u64::try_from(millis).unwrap_or(u64::MAX),
            course_id,
        });
    }

    fn remove(&mut self, challenge: Uuid, action: AuditAction) {
        if let Some(grant) = self.peers.remove(&challenge).and_then(|peer| peer.grant) {
            self.record(action, Some(grant.course_id));
        }
    }
}

type Result<T> = std::result::Result<T, AppError>;

impl Connections {
    fn lock(&self) -> Result<std::sync::MutexGuard<'_, State>> {
        self.0
            .lock()
            .map_err(|_| AppError::new(ErrorCode::OwnerUnavailable))
    }

    pub(crate) fn status(&self) -> Result<Status> {
        let mut state = self.lock()?;
        purge(&mut state);
        let mut peers: Vec<_> = state
            .peers
            .iter()
            .map(|(id, peer)| PeerStatus {
                challenge: *id,
                pid: peer.pid,
                course_id: peer.grant.as_ref().map(|grant| grant.course_id),
                permissions: peer.grant.as_ref().map(|grant| grant.permissions),
            })
            .collect();
        peers.sort_by_key(|peer| peer.challenge);
        Ok(Status {
            enabled: state.broker.is_some(),
            peers,
            recent_events: state.audit.iter().cloned().collect(),
        })
    }

    pub(crate) fn set_enabled(&self, enabled: bool) -> Result<()> {
        let mut state = self.lock()?;
        if !enabled {
            state.peers.clear();
            if state.broker.take().is_some() {
                state.record(AuditAction::Disabled, None);
            }
        } else if state.broker.is_none() {
            let socket = Socket::bind().map_err(|_| AppError::new(ErrorCode::OwnerUnavailable))?;
            let runtime = tauri::async_runtime::handle();
            let _entered = runtime.inner().enter();
            let listener = UnixListener::from_std(socket.listener.try_clone()?)
                .map_err(|_| AppError::new(ErrorCode::OwnerUnavailable))?;
            let (stop, stopped) = watch::channel(false);
            let (finished, closed) = oneshot::channel();
            state.broker = Some(stop);
            state.closed = Some(closed);
            state.record(AuditAction::Enabled, None);
            let connections = self.clone();
            tauri::async_runtime::spawn(async move {
                connections.accept(listener, stopped).await;
                drop(socket);
                let _ = finished.send(());
            });
        }
        Ok(())
    }

    pub(crate) async fn shutdown(&self) -> Result<()> {
        self.set_enabled(false)?;
        let closed = self.lock()?.closed.take();
        if let Some(closed) = closed {
            let _ = timeout(Duration::from_secs(2), closed).await;
        }
        Ok(())
    }

    pub(crate) fn approve(
        &self,
        manager: &CourseManager,
        challenge: Uuid,
        course_id: Uuid,
        permissions: ReadPermissions,
    ) -> Result<()> {
        let mut state = self.lock()?;
        purge(&mut state);
        let peer = state
            .peers
            .get_mut(&challenge)
            .filter(|peer| peer.grant.is_none())
            .ok_or_else(denied)?;
        let reads = CourseReads::approve(manager, course_id, permissions)?;
        peer.grant = Some(Grant {
            course_id,
            permissions,
            reads,
        });
        peer.expires = Instant::now() + GRANT_WINDOW;
        state.record(AuditAction::Approved, Some(course_id));
        Ok(())
    }

    pub(crate) fn revoke(&self, challenge: Uuid) -> Result<()> {
        self.lock()?.remove(challenge, AuditAction::Revoked);
        Ok(())
    }

    async fn accept(&self, listener: UnixListener, mut stopped: watch::Receiver<bool>) {
        loop {
            let stream = tokio::select! {
                biased;
                _ = stopped.changed() => return,
                result = listener.accept() => match result {
                    Ok((stream, _)) => stream,
                    Err(_) => return,
                }
            };
            let Ok(credential) = stream.peer_cred() else {
                continue;
            };
            if credential.uid() != wire::uid() {
                continue;
            }
            let Ok((peer, revoked)) = self.register(credential.pid()) else {
                continue;
            };
            tauri::async_runtime::spawn(peer.serve(stream, revoked));
        }
    }

    fn register(&self, pid: Option<i32>) -> Result<(Connection, watch::Receiver<bool>)> {
        let mut state = self.lock()?;
        purge(&mut state);
        if state.broker.is_none() || state.peers.len() >= MAX_CONNECTIONS {
            return Err(denied());
        }
        let challenge = Uuid::new_v4();
        let (revoked, receiver) = watch::channel(false);
        state.peers.insert(
            challenge,
            Peer {
                pid,
                expires: Instant::now() + PAIR_WINDOW,
                grant: None,
                _revoked: revoked,
            },
        );
        Ok((
            Connection {
                connections: self.clone(),
                challenge,
            },
            receiver,
        ))
    }
}

fn purge(state: &mut State) {
    let now = Instant::now();
    let expired: Vec<_> = state
        .peers
        .iter()
        .filter(|(_, peer)| peer.expires <= now)
        .map(|(id, _)| *id)
        .collect();
    for challenge in expired {
        state.remove(challenge, AuditAction::Expired);
    }
}

fn denied() -> AppError {
    AppError::new(ErrorCode::PermissionDenied)
}

struct Connection {
    connections: Connections,
    challenge: Uuid,
}

impl Connection {
    async fn serve(self, mut stream: UnixStream, mut revoked: watch::Receiver<bool>) {
        loop {
            let request = tokio::select! {
                biased;
                _ = revoked.changed() => return,
                result = self.receive(&mut stream) => match result {
                    Ok(request) => request,
                    Err(_) => return,
                },
            };
            // Read the grant again after the wait for input. Approval can occur while idle.
            let Ok((expires, reads)) = self.access() else {
                return;
            };
            let reply = tokio::select! {
                biased;
                _ = revoked.changed() => return,
                reply = timeout(IO_TIMEOUT, self.read(request, reads)) => match reply {
                    Ok(reply) => reply,
                    Err(_) => return,
                }
            };
            let mut bytes = serde_json::to_vec(&reply).unwrap_or_default();
            if bytes.len() > REPLY_LIMIT {
                bytes = serde_json::to_vec(&error(
                    "budget_exceeded",
                    "Read a smaller page or a narrower resource.",
                ))
                .unwrap_or_default();
            }
            if bytes.is_empty() || self.access().is_err() {
                return;
            }
            let deadline = expires.min(Instant::now() + IO_TIMEOUT);
            tokio::select! {
                biased;
                _ = revoked.changed() => return,
                result = tokio::time::timeout_at(deadline.into(), self.send(&stream, &bytes)) => {
                    if !matches!(result, Ok(Ok(()))) {
                        return;
                    }
                }
            }
        }
    }

    async fn receive(&self, stream: &mut UnixStream) -> io::Result<Request> {
        let mut frame = std::pin::pin!(wire::receive(stream, REQUEST_LIMIT));
        loop {
            let (expires, _) = self.access().map_err(|_| io::ErrorKind::PermissionDenied)?;
            if let Ok(result) = tokio::time::timeout_at(expires.into(), &mut frame).await {
                return result;
            }
            // A user may have approved the bridge while it was waiting for input.
            // Keep the partial frame and consult the new deadline before continuing.
        }
    }

    async fn send(&self, stream: &UnixStream, bytes: &[u8]) -> io::Result<()> {
        let mut frame = Vec::with_capacity(4 + bytes.len());
        frame.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
        frame.extend_from_slice(bytes);
        let mut remaining = frame.as_slice();
        while !remaining.is_empty() {
            stream.writable().await?;
            let state = self
                .connections
                .lock()
                .map_err(|_| io::ErrorKind::PermissionDenied)?;
            if state
                .peers
                .get(&self.challenge)
                .is_none_or(|peer| peer.expires <= Instant::now())
            {
                return Err(io::ErrorKind::PermissionDenied.into());
            }
            // Each nonblocking write and desktop revocation share this lock.
            // A partially written frame is unusable to the bridge after revocation.
            match stream.try_write(remaining) {
                Ok(0) => return Err(io::ErrorKind::WriteZero.into()),
                Ok(sent) => remaining = &remaining[sent..],
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }

    fn access(&self) -> Result<(Instant, Option<CourseReads>)> {
        let mut state = self.connections.lock()?;
        purge(&mut state);
        let peer = state.peers.get(&self.challenge).ok_or_else(denied)?;
        Ok((
            peer.expires,
            peer.grant.as_ref().map(|grant| grant.reads.clone()),
        ))
    }

    async fn read(&self, request: Request, reads: Option<CourseReads>) -> Reply {
        match (request, reads) {
            (Request::Status, Some(_)) => Reply::Success {
                result: json!({"connection": "approved"}),
            },
            (Request::Status, None) => Reply::Success {
                result: json!({
                    "connection": "pending",
                    "challenge": self.challenge,
                    "action": "Open Connections in Memoquill, compare the challenge and approve the chosen course."
                }),
            },
            (Request::Read { request }, Some(reads)) => match reads.read(request).await {
                Ok(result) => Reply::Success { result },
                Err(error) => {
                    let code = serde_json::to_value(error.code).unwrap_or_default();
                    Reply::Error {
                        code: code.as_str().unwrap_or("unavailable").into(),
                        message: "Memoquill did not disclose the requested data.".into(),
                    }
                }
            },
            (Request::Draft { request }, Some(reads)) => {
                match reads.draft(request).await {
                    Ok(result) => Reply::Success { result },
                    Err(error) => {
                        let code = serde_json::to_value(error.code).unwrap_or_default();
                        Reply::Error {
                        code: code.as_str().unwrap_or("unavailable").into(),
                        message: "The draft request did not complete. Inspect the draft before retrying.".into(),
                    }
                    }
                }
            }
            (Request::CourseDraft { request }, Some(reads)) => {
                match reads.course_draft(request).await {
                    Ok(result) => Reply::Success { result },
                    Err(error) => {
                        let code = serde_json::to_value(error.code).unwrap_or_default();
                        Reply::Error {
                        code: code.as_str().unwrap_or("unavailable").into(),
                        message: "The course draft request did not complete. Inspect the draft before retrying.".into(),
                    }
                    }
                }
            }
            (Request::Read { .. } | Request::Draft { .. } | Request::CourseDraft { .. }, None) => {
                error(
                    "permission_denied",
                    "Call memoquill_connection and approve this connection in Memoquill.",
                )
            }
        }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        if let Ok(mut state) = self.connections.lock() {
            state.remove(self.challenge, AuditAction::Disconnected);
        }
    }
}

pub(super) fn error(code: &str, message: &str) -> Reply {
    Reply::Error {
        code: code.into(),
        message: message.into(),
    }
}

struct Socket {
    listener: std::os::unix::net::UnixListener,
    path: PathBuf,
    identity: (u64, u64),
    _lock: std::os::fd::OwnedFd,
}

impl Socket {
    fn bind() -> io::Result<Self> {
        let path = wire::socket_path()?;
        let parent = path.parent().ok_or(io::ErrorKind::InvalidInput)?;
        match fs::DirBuilder::new().mode(0o700).create(parent) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
        wire::verify_dir(parent)?;
        use rustix::fs::{FlockOperation, Mode, OFlags};
        let lock = rustix::fs::open(
            parent.join("owner.lock"),
            OFlags::CREATE | OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOFOLLOW | OFlags::NONBLOCK,
            Mode::RUSR | Mode::WUSR,
        )?;
        let meta = rustix::fs::fstat(&lock)?;
        if !rustix::fs::FileType::from_raw_mode(meta.st_mode).is_file()
            || meta.st_uid != wire::uid()
            || meta.st_mode & 0o077 != 0
        {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        rustix::fs::flock(&lock, FlockOperation::NonBlockingLockExclusive)?;
        if let Ok(meta) = fs::symlink_metadata(&path) {
            if !meta.file_type().is_socket() || meta.uid() != wire::uid() {
                return Err(io::ErrorKind::PermissionDenied.into());
            }
            match std::os::unix::net::UnixStream::connect(&path) {
                Err(error) if error.kind() == io::ErrorKind::ConnectionRefused => {
                    fs::remove_file(&path)?
                }
                _ => return Err(io::ErrorKind::AddrInUse.into()),
            }
        }
        let listener = std::os::unix::net::UnixListener::bind(&path)?;
        let meta = fs::symlink_metadata(&path)?;
        let socket = Self {
            listener,
            path,
            identity: (meta.dev(), meta.ino()),
            _lock: lock,
        };
        fs::set_permissions(&socket.path, fs::Permissions::from_mode(0o600))?;
        socket.listener.set_nonblocking(true)?;
        Ok(socket)
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        if fs::symlink_metadata(&self.path)
            .is_ok_and(|meta| (meta.dev(), meta.ino()) == self.identity)
        {
            let _ = fs::remove_file(&self.path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::mcp_reads::CourseRead;

    #[tokio::test]
    async fn mcp_grant_is_bound_to_connection_and_course() {
        let temp = tempfile::tempdir().unwrap();
        let manager = CourseManager::open(temp.path().to_owned()).unwrap();
        let one = manager.list().unwrap().selected_course_id;
        let two = Uuid::new_v4();
        manager.create(two, "Second course".into()).unwrap();
        manager.select(one).unwrap();
        let connections = Connections::default();
        connections.lock().unwrap().broker = Some(watch::channel(false).0);
        let (approved, mut revoked) = connections.register(Some(111)).unwrap();
        let (other, _other_revoked) = connections.register(Some(222)).unwrap();
        let content_only = ReadPermissions {
            content: true,
            progress: false,
            sources: false,
            originals: false,
            drafts: false,
            delegate_additions: false,
            responses: false,
        };

        assert!(approved.access().unwrap().1.is_none());
        assert!(
            connections
                .approve(&manager, approved.challenge, two, content_only)
                .is_err()
        );
        connections
            .approve(&manager, approved.challenge, one, content_only)
            .unwrap();
        manager.select(two).unwrap();

        let reads = approved.access().unwrap().1.unwrap();
        assert_eq!(
            reads.read(CourseRead::Capabilities).await.unwrap()["courseId"],
            json!(one)
        );
        assert_eq!(
            reads
                .read(CourseRead::Progress {
                    id: Uuid::new_v4(),
                    offset: 0,
                    include_responses: false
                })
                .await
                .unwrap_err()
                .code,
            ErrorCode::PermissionDenied,
        );
        assert!(other.access().unwrap().1.is_none());
        assert!(
            connections
                .approve(&manager, approved.challenge, two, content_only)
                .is_err()
        );
        connections.revoke(approved.challenge).unwrap();
        assert!(revoked.changed().await.is_err());
        assert!(approved.access().is_err());
        assert!(other.access().is_ok());
        manager.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn mcp_expired_pairing_cannot_be_approved() {
        let temp = tempfile::tempdir().unwrap();
        let manager = CourseManager::open(temp.path().to_owned()).unwrap();
        let course = manager.list().unwrap().selected_course_id;
        let connections = Connections::default();
        connections.lock().unwrap().broker = Some(watch::channel(false).0);
        let (connection, mut revoked) = connections.register(None).unwrap();
        connections
            .lock()
            .unwrap()
            .peers
            .get_mut(&connection.challenge)
            .unwrap()
            .expires = Instant::now();
        assert!(
            connections
                .approve(
                    &manager,
                    connection.challenge,
                    course,
                    ReadPermissions {
                        content: true,
                        progress: true,
                        sources: false,
                        originals: false,
                        drafts: false,
                        delegate_additions: false,
                        responses: false,
                    }
                )
                .is_err()
        );
        assert!(revoked.changed().await.is_err());
        manager.shutdown().await.unwrap();
    }
}
