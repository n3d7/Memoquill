use super::{
    database::{Database, SCHEMA_VERSION},
    error::Result,
    *,
};
use std::{io::Read, path::PathBuf};
use tokio::sync::{mpsc, oneshot};

type Reply<T> = oneshot::Sender<Result<T>>;
const QUEUE_CAPACITY: usize = 64;

enum Request {
    Status(Reply<WorkspaceStatus>),
    Save(RequestContext, SavePreferences, Reply<PreferencesSnapshot>),
    Receipt(RequestContext, OperationId, Reply<Option<OperationReceipt>>),
    Asset(
        RequestContext,
        RegisterAsset,
        Box<dyn Read + Send>,
        u64,
        Reply<AssetInfo>,
    ),
    Reopen(Reply<WorkspaceStatus>),
    Shutdown(Reply<()>),
}

/// Typed application boundary; no connection, SQL, caller path or principal payload.
#[derive(Clone)]
pub struct ApplicationService {
    sender: mpsc::Sender<Request>,
}
impl ApplicationService {
    /// The native adapter supplies the managed root. Opening/auditing happens off the UI thread.
    pub fn start(root: PathBuf) -> Result<Self> {
        let (sender, receiver) = mpsc::channel(QUEUE_CAPACITY);
        std::thread::Builder::new()
            .name("memoquill-storage".into())
            .spawn(move || worker(root, receiver))?;
        Ok(Self { sender })
    }
    pub async fn status(&self, context: RequestContext) -> Result<WorkspaceStatus> {
        self.request(context, Request::Status, false).await
    }
    pub async fn save_preferences(
        &self,
        context: RequestContext,
        request: SavePreferences,
    ) -> Result<PreferencesSnapshot> {
        self.request(
            context,
            |reply| Request::Save(context, request, reply),
            true,
        )
        .await
    }
    pub async fn operation_receipt(
        &self,
        context: RequestContext,
        id: OperationId,
    ) -> Result<Option<OperationReceipt>> {
        self.request(context, |reply| Request::Receipt(context, id, reply), false)
            .await
    }
    /// Rust-service seam for future ingestion. This is deliberately not a native IPC command.
    pub async fn register_asset(
        &self,
        context: RequestContext,
        request: RegisterAsset,
        reader: Box<dyn Read + Send>,
        limit: u64,
    ) -> Result<AssetInfo> {
        self.request(
            context,
            |reply| Request::Asset(context, request, reader, limit, reply),
            true,
        )
        .await
    }
    pub async fn reopen(&self, context: RequestContext) -> Result<WorkspaceStatus> {
        self.request(context, Request::Reopen, false).await
    }
    pub async fn shutdown(&self) -> Result<()> {
        let (reply, response) = oneshot::channel();
        self.sender
            .send(Request::Shutdown(reply))
            .await
            .map_err(|_| AppError::new(ErrorCode::OwnerUnavailable))?;
        response
            .await
            .map_err(|_| AppError::new(ErrorCode::OwnerUnavailable))?
    }
    async fn request<T>(
        &self,
        context: RequestContext,
        build: impl FnOnce(Reply<T>) -> Request,
        mutation: bool,
    ) -> Result<T> {
        let (reply, response) = oneshot::channel();
        context.authorize()?;
        self.sender.try_send(build(reply)).map_err(queue_error)?;
        response.await.map_err(|_| {
            AppError::new(if mutation {
                ErrorCode::OutcomeUnconfirmed
            } else {
                ErrorCode::OwnerUnavailable
            })
        })?
    }
}

fn queue_error(error: mpsc::error::TrySendError<Request>) -> AppError {
    AppError::new(match error {
        mpsc::error::TrySendError::Full(_) => ErrorCode::Busy,
        mpsc::error::TrySendError::Closed(_) => ErrorCode::OwnerUnavailable,
    })
}
struct Store {
    root: PathBuf,
    database: Result<Database>,
}
impl Store {
    fn access<T>(&mut self, action: impl FnOnce(&mut Database) -> Result<T>) -> Result<T> {
        let degraded = self
            .database
            .as_ref()
            .is_ok_and(|db| db.recovery.blocks_writes());
        let result = match &mut self.database {
            Ok(db) => action(db),
            Err(error) => Err(*error),
        };
        if let Err(error) = &result
            && (error.code == ErrorCode::OutcomeUnconfirmed
                || (error.code == ErrorCode::RecoveryRequired && !degraded))
        {
            self.database = Err(*error); // Close before another operation; caller must explicitly reopen.
        }
        result
    }
    fn handle(&mut self, request: Request) {
        match request {
            Request::Status(reply) => {
                let _ = reply.send(self.access(status));
            }
            Request::Save(context, request, reply) => {
                let _ = reply.send(self.access(|db| db.save_preferences(context, &request)));
            }
            Request::Receipt(context, id, reply) => {
                let _ = reply.send(self.access(|db| db.operation_receipt(context, &id)));
            }
            Request::Asset(context, request, mut reader, limit, reply) => {
                let _ = reply.send(
                    self.access(|db| db.register_asset(context, &request, &mut reader, limit)),
                );
            }
            Request::Reopen(reply) => {
                self.database = Err(AppError::new(ErrorCode::OwnerUnavailable));
                self.database = Database::open(&self.root);
                let _ = reply.send(self.access(status));
            }
            Request::Shutdown(_) => unreachable!("shutdown is owned by the worker loop"),
        }
    }
}
fn status(db: &mut Database) -> Result<WorkspaceStatus> {
    Ok(WorkspaceStatus {
        contract_version: 1,
        schema_version: SCHEMA_VERSION,
        preferences: db.preferences()?,
        recovery: db.recovery_report(),
        sqlite_version: rusqlite::version().to_owned(),
    })
}
fn worker(root: PathBuf, mut receiver: mpsc::Receiver<Request>) {
    let database = Database::open(&root);
    let mut store = Store { root, database };
    let mut shutdown = Vec::new();
    while let Some(request) = receiver.blocking_recv() {
        match request {
            Request::Shutdown(reply) => {
                receiver.close();
                shutdown.push(reply);
            }
            request => store.handle(request),
        }
    }
    drop(store); // SQLite closes, then the retained file lock drops, before acknowledgement.
    for reply in shutdown {
        let _ = reply.send(Ok(()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::owner::Owner;

    #[test]
    fn service_owns_one_workspace_and_shutdown_releases_its_lock() {
        tauri::async_runtime::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path().join("workspace");
            let service = ApplicationService::start(root.clone()).unwrap();
            let status = service.status(RequestContext::local_ui()).await.unwrap();
            assert_eq!(status.preferences.revision, 0);
            assert_eq!(
                Owner::acquire(&root).err().unwrap().code,
                ErrorCode::AlreadyOpen
            );
            service.shutdown().await.unwrap();
            let _owner = Owner::acquire(&root).unwrap();
            assert!(service.status(RequestContext::local_ui()).await.is_err());
        });
    }
    #[test]
    fn unverified_context_cannot_read_workspace_status() {
        tauri::async_runtime::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path().join("workspace");
            let service = ApplicationService::start(root).unwrap();
            assert_eq!(
                service
                    .status(RequestContext::unverified_external())
                    .await
                    .unwrap_err()
                    .code,
                ErrorCode::PermissionDenied
            );
            service.shutdown().await.unwrap();
        });
    }

    #[test]
    fn accepted_save_outlives_its_reply_and_shutdown_drains_the_queue() {
        tauri::async_runtime::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path().join("workspace");
            let service = ApplicationService::start(root.clone()).unwrap();
            let initial = service.status(RequestContext::local_ui()).await.unwrap();
            let request = SavePreferences {
                contract_version: 1,
                workspace_id: initial.preferences.workspace_id,
                operation_id: OperationId::generate(),
                expected_revision: 0,
                preferences: ReadingPreferences {
                    theme: Theme::Dark,
                    text_scale: TextScale::try_from(125).unwrap(),
                },
            };
            let (reply, response) = oneshot::channel();
            drop(response);
            service
                .sender
                .try_send(Request::Save(
                    RequestContext::local_ui(),
                    request.clone(),
                    reply,
                ))
                .unwrap();
            service.shutdown().await.unwrap();
            let reopened = ApplicationService::start(root).unwrap();
            let status = reopened.status(RequestContext::local_ui()).await.unwrap();
            assert_eq!(status.preferences.preferences, request.preferences);
            assert_eq!(status.preferences.revision, 1);
            assert_eq!(
                reopened
                    .save_preferences(RequestContext::local_ui(), request.clone())
                    .await
                    .unwrap(),
                status.preferences
            );
            assert!(
                reopened
                    .operation_receipt(RequestContext::local_ui(), request.operation_id)
                    .await
                    .unwrap()
                    .is_some()
            );
            assert_eq!(
                reopened
                    .reopen(RequestContext::local_ui())
                    .await
                    .unwrap()
                    .preferences,
                status.preferences
            );
            reopened.shutdown().await.unwrap();
        });
    }

    #[test]
    fn bounded_admission_rejects_full_and_closed_owner_without_accepting_a_request() {
        tauri::async_runtime::block_on(async {
            let (sender, mut receiver) = mpsc::channel(1);
            let service = ApplicationService { sender };
            let (reply, _response) = oneshot::channel();
            service.sender.try_send(Request::Status(reply)).unwrap();
            assert_eq!(
                service
                    .status(RequestContext::local_ui())
                    .await
                    .unwrap_err()
                    .code,
                ErrorCode::Busy
            );
            assert_eq!(receiver.len(), 1);
            receiver.close();
            drop(receiver);
            assert_eq!(
                service
                    .status(RequestContext::local_ui())
                    .await
                    .unwrap_err()
                    .code,
                ErrorCode::OwnerUnavailable
            );
        });
    }

    #[test]
    fn failed_open_can_be_retried_without_replacing_unknown_data() {
        tauri::async_runtime::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path().join("workspace");
            drop(Owner::acquire(&root).unwrap());
            let path = root.join("memoquill.sqlite3");
            std::fs::write(&path, b"unknown").unwrap();
            let service = ApplicationService::start(root.clone()).unwrap();
            assert!(service.status(RequestContext::local_ui()).await.is_err());
            assert!(
                service
                    .reopen(RequestContext::unverified_external())
                    .await
                    .is_err()
            );
            assert!(service.reopen(RequestContext::local_ui()).await.is_err());
            assert_eq!(std::fs::read(&path).unwrap(), b"unknown");
            service.shutdown().await.unwrap();
        });
    }

    #[test]
    fn recovery_guard_keeps_preferences_and_receipts_readable_after_a_denied_save() {
        tauri::async_runtime::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path().join("workspace");
            let service = ApplicationService::start(root.clone()).unwrap();
            let initial = service.status(RequestContext::local_ui()).await.unwrap();
            let request = RegisterAsset {
                contract_version: 1,
                workspace_id: initial.preferences.workspace_id.clone(),
                operation_id: OperationId::generate(),
            };
            let info = service
                .register_asset(
                    RequestContext::local_ui(),
                    request.clone(),
                    Box::new(std::io::Cursor::new(b"original")),
                    1024,
                )
                .await
                .unwrap();
            service.shutdown().await.unwrap();
            std::fs::remove_file(super::super::assets::asset_path(&root, &info.digest)).unwrap();
            let reopened = ApplicationService::start(root).unwrap();
            let status = reopened.status(RequestContext::local_ui()).await.unwrap();
            assert_eq!(status.recovery.missing_assets, 1);
            let save = SavePreferences {
                contract_version: 1,
                workspace_id: initial.preferences.workspace_id,
                operation_id: OperationId::generate(),
                expected_revision: 0,
                preferences: ReadingPreferences::default(),
            };
            assert_eq!(
                reopened
                    .save_preferences(RequestContext::local_ui(), save)
                    .await
                    .unwrap_err()
                    .code,
                ErrorCode::RecoveryRequired
            );
            assert!(
                reopened
                    .operation_receipt(RequestContext::local_ui(), request.operation_id)
                    .await
                    .unwrap()
                    .is_some()
            );
            assert_eq!(
                reopened.status(RequestContext::local_ui()).await.unwrap(),
                status
            );
            reopened.shutdown().await.unwrap();
        });
    }
    #[test]
    fn accepted_mutation_with_a_disappearing_worker_has_an_unconfirmed_outcome() {
        tauri::async_runtime::block_on(async {
            for kind in ["preferences", "asset"] {
                let (sender, mut receiver) = mpsc::channel(1);
                let service = ApplicationService { sender };
                let accepted = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
                let observed = accepted.clone();
                let worker = std::thread::spawn(move || {
                    let message = receiver.blocking_recv();
                    observed.store(message.is_some(), std::sync::atomic::Ordering::Release);
                    drop(message);
                });
                let workspace_id = WorkspaceId::generate();
                let operation_id = OperationId::generate();
                let error = if kind == "preferences" {
                    service
                        .save_preferences(
                            RequestContext::local_ui(),
                            SavePreferences {
                                contract_version: 1,
                                workspace_id,
                                operation_id,
                                expected_revision: 0,
                                preferences: ReadingPreferences::default(),
                            },
                        )
                        .await
                        .unwrap_err()
                } else {
                    service
                        .register_asset(
                            RequestContext::local_ui(),
                            RegisterAsset {
                                contract_version: 1,
                                workspace_id,
                                operation_id,
                            },
                            Box::new(std::io::Cursor::new(b"source")),
                            1024,
                        )
                        .await
                        .unwrap_err()
                };
                assert_eq!(error.code, ErrorCode::OutcomeUnconfirmed);
                worker.join().unwrap();
                assert!(accepted.load(std::sync::atomic::Ordering::Acquire));
            }
        });
    }
    #[test]
    fn uncertain_commit_closes_the_owner_until_reopen_and_preserves_its_receipt() {
        use super::super::{
            database::Database,
            faults::{self, Phase},
        };
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        let database = Database::open(&root).unwrap();
        let request = RegisterAsset {
            contract_version: 1,
            workspace_id: database.id.clone(),
            operation_id: OperationId::generate(),
        };
        let mut store = Store {
            root: root.clone(),
            database: Ok(database),
        };
        let result = faults::using(
            |point| {
                if point == Phase::AfterCommit {
                    Err(AppError::new(ErrorCode::StorageFailure))
                } else {
                    Ok(())
                }
            },
            || {
                store.access(|db| {
                    db.register_asset(
                        RequestContext::local_ui(),
                        &request,
                        &mut std::io::Cursor::new(b"source"),
                        1024,
                    )
                })
            },
        );
        assert_eq!(result.unwrap_err().code, ErrorCode::OutcomeUnconfirmed);
        drop(Owner::acquire(&root).unwrap());
        assert_eq!(
            store.access(status).unwrap_err().code,
            ErrorCode::OutcomeUnconfirmed
        );
        let (reply, response) = oneshot::channel();
        store.handle(Request::Reopen(reply));
        let reopened = tauri::async_runtime::block_on(response).unwrap().unwrap();
        assert_eq!(reopened.contract_version, 1);
        let receipt = store
            .access(|db| db.operation_receipt(RequestContext::local_ui(), &request.operation_id))
            .unwrap()
            .unwrap();
        assert!(matches!(
            receipt.result,
            ReceiptResult::Applied(OperationOutcome::Asset(_))
        ));
    }
}
