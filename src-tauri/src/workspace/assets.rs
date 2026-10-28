use super::{
    database::Database,
    error::Result,
    faults::{self, Phase},
    operations, owner, *,
};
use rusqlite::{TransactionBehavior, params};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

struct Staged {
    path: PathBuf,
    info: AssetInfo,
}

impl Database {
    pub(super) fn register_asset(
        &mut self,
        context: RequestContext,
        request: &RegisterAsset,
        reader: &mut dyn Read,
        limit: u64,
    ) -> Result<AssetInfo> {
        self.authorize_workspace(context, &request.workspace_id)?;
        validate_request(request, limit)?;
        let staged = stage(self.owner.root(), reader, limit)?;
        let hash = operations::request_hash("asset", &(request, &staged.info))?;
        if let Some(receipt) =
            operations::exact_receipt(&self.connection, &self.id, &request.operation_id, &hash)?
        {
            return replay_asset(self.owner.root(), staged, receipt);
        }
        self.connection.execute("INSERT INTO operations(id,workspace_id,principal,request_hash,kind,state) VALUES (?1,?2,'local-ui',?3,'asset','prepared')",params![request.operation_id.as_str(),self.id.as_str(),hash])?;
        let result = self.apply_asset(request, &staged);
        if let Err(error) = &result
            && error.code != ErrorCode::OutcomeUnconfirmed
            && interrupt(&self.connection, &request.operation_id, *error).is_err()
        {
            return Err(AppError::new(ErrorCode::RecoveryRequired));
        }
        result
    }
    fn apply_asset(&mut self, request: &RegisterAsset, staged: &Staged) -> Result<AssetInfo> {
        publish(self.owner.root(), staged)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute("INSERT INTO assets(digest,byte_length,registered_by) VALUES (?1,?2,?3) ON CONFLICT(digest) DO NOTHING",params![staged.info.digest.as_str(),staged.info.byte_length as i64,request.operation_id.as_str()])?;
        let receipt = operations::applied(
            request.operation_id.clone(),
            OperationOutcome::Asset(staged.info.clone()),
        );
        let changed = tx.execute(
            "UPDATE operations SET state='applied',result=?1 WHERE id=?2 AND state='prepared'",
            params![operations::json(&receipt)?, request.operation_id.as_str()],
        )?;
        if changed != 1 {
            return Err(AppError::new(ErrorCode::RecoveryRequired));
        }
        faults::check(Phase::BeforeCommit)?;
        operations::durable_commit(tx)?;
        faults::check(Phase::AfterCommit)
            .map_err(|_| AppError::new(ErrorCode::OutcomeUnconfirmed))?;
        Ok(staged.info.clone())
    }
}

fn validate_request(request: &RegisterAsset, limit: u64) -> Result<()> {
    if request.contract_version != 1 || !(1..=i64::MAX as u64).contains(&limit) {
        return Err(AppError::new(ErrorCode::InvalidRequest));
    }
    Ok(())
}
fn stage(root: &Path, reader: &mut dyn Read, limit: u64) -> Result<Staged> {
    let path = root
        .join("staging")
        .join(format!("{}.part", OperationId::generate().as_str()));
    let mut file = owner::create_private_file(&path)?;
    let info = copy_hash(reader, &mut file, limit)?;
    faults::check(Phase::FileSync)?;
    file.sync_all()?;
    Ok(Staged { path, info })
}
fn copy_hash(reader: &mut dyn Read, writer: &mut dyn Write, limit: u64) -> Result<AssetInfo> {
    let mut hash = Sha256::new();
    let mut length = 0_u64;
    let mut buffer = [0_u8; 16384];
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        length = length
            .checked_add(n as u64)
            .ok_or(AppError::new(ErrorCode::InvalidRequest))?;
        if length > limit {
            return Err(AppError::new(ErrorCode::InvalidRequest));
        }
        faults::check(Phase::Write)?;
        writer.write_all(&buffer[..n])?;
        hash.update(&buffer[..n]);
    }
    Ok(AssetInfo {
        digest: AssetDigest::try_from(format!("{:x}", hash.finalize()))?,
        byte_length: length,
    })
}
pub(super) fn asset_path(root: &Path, digest: &AssetDigest) -> PathBuf {
    let value = digest.as_str();
    root.join("assets/sha256")
        .join(&value[..2])
        .join(&value[2..])
}
fn publish(root: &Path, staged: &Staged) -> Result<()> {
    let destination = publication_destination(root, &staged.info.digest)?;
    if std::fs::symlink_metadata(&destination).is_ok() {
        verify(root, &staged.info)?;
        remove_staged(&staged.path)?;
    } else {
        faults::check(Phase::Publish)?;
        owner::publish_no_replace(&staged.path, &destination)?;
    }
    sync_publication(root, &destination)
}
fn publication_destination(root: &Path, digest: &AssetDigest) -> Result<PathBuf> {
    let destination = asset_path(root, digest);
    let parent = destination
        .parent()
        .ok_or(AppError::new(ErrorCode::InvalidRequest))?;
    owner::create_private_tree(parent)?;
    owner::private_directory(parent)?;
    Ok(destination)
}
fn sync_publication(root: &Path, destination: &Path) -> Result<()> {
    let parent = destination
        .parent()
        .ok_or(AppError::new(ErrorCode::InvalidRequest))?;
    faults::check(Phase::DirectorySync)?;
    owner::sync_directory(parent)?;
    owner::sync_directory(&root.join("assets/sha256"))?;
    owner::sync_directory(&root.join("staging"))
}
pub(super) fn verify(root: &Path, info: &AssetInfo) -> Result<()> {
    let path = asset_path(root, &info.digest);
    owner::private_directory(
        path.parent()
            .ok_or(AppError::new(ErrorCode::RecoveryRequired))?,
    )?;
    let flags = rustix::fs::OFlags::RDONLY
        | rustix::fs::OFlags::NOFOLLOW
        | rustix::fs::OFlags::CLOEXEC
        | rustix::fs::OFlags::NONBLOCK;
    let fd = rustix::fs::open(&path, flags, rustix::fs::Mode::empty())
        .map_err(|_| AppError::new(ErrorCode::RecoveryRequired))?;
    let mut file = File::from(fd);
    let meta = file.metadata()?;
    if !meta.is_file() || meta.len() != info.byte_length {
        return Err(AppError::new(ErrorCode::RecoveryRequired));
    }
    let actual = copy_hash(&mut file, &mut std::io::sink(), info.byte_length)
        .map_err(|_| AppError::new(ErrorCode::RecoveryRequired))?;
    if actual != *info {
        return Err(AppError::new(ErrorCode::RecoveryRequired));
    }
    Ok(())
}
fn replay_asset(root: &Path, staged: Staged, receipt: OperationReceipt) -> Result<AssetInfo> {
    match receipt.result {
        ReceiptResult::Applied(OperationOutcome::Asset(info)) if info == staged.info => {
            verify(root, &info)?;
            remove_staged(&staged.path)?;
            Ok(info)
        }
        ReceiptResult::Interrupted(code) => Err(AppError::new(code)),
        _ => Err(AppError::new(ErrorCode::RecoveryRequired)),
    }
}
fn remove_staged(path: &Path) -> Result<()> {
    std::fs::remove_file(path)?;
    owner::sync_directory(
        path.parent()
            .ok_or(AppError::new(ErrorCode::InvalidRequest))?,
    )?;
    Ok(())
}
pub(super) fn interrupt(
    connection: &rusqlite::Connection,
    id: &OperationId,
    error: AppError,
) -> Result<()> {
    let receipt = OperationReceipt {
        operation_id: id.clone(),
        result: ReceiptResult::Interrupted(error.code),
    };
    connection.execute("UPDATE operations SET state='interrupted',result=?1,error=?2 WHERE id=?3 AND state='prepared'",params![operations::json(&receipt)?,operations::json(&error.code)?,id.as_str()])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn request(db: &Database) -> RegisterAsset {
        RegisterAsset {
            contract_version: 1,
            workspace_id: db.id.clone(),
            operation_id: OperationId::generate(),
        }
    }
    #[test]
    fn invalid_digest_cannot_deserialize_as_an_asset_reference() {
        assert!(
            serde_json::from_value::<AssetInfo>(
                serde_json::json!({"digest":"../outside","byteLength":4})
            )
            .is_err()
        );
    }
    #[test]
    fn source_bytes_publish_once_and_receipt_survives_restart() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        let mut db = Database::open(&root).unwrap();
        let req = request(&db);
        let data = b"# Original\nUnicode: caf\xc3\xa9; code is never executed.\n";
        let info = db
            .register_asset(
                RequestContext::local_ui(),
                &req,
                &mut Cursor::new(data),
                1024,
            )
            .unwrap();
        let path = root
            .join("assets/sha256")
            .join(&info.digest.as_str()[..2])
            .join(&info.digest.as_str()[2..]);
        assert_eq!(std::fs::read(&path).unwrap(), data);
        drop(db);
        let mut reopened = Database::open(&root).unwrap();
        assert_eq!(
            reopened
                .register_asset(
                    RequestContext::local_ui(),
                    &req,
                    &mut Cursor::new(data),
                    1024
                )
                .unwrap(),
            info
        );
        assert!(
            reopened
                .operation_receipt(RequestContext::local_ui(), &req.operation_id)
                .unwrap()
                .is_some()
        );
        assert_eq!(
            reopened
                .connection
                .query_row("SELECT count(*) FROM assets", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            1
        );
    }
    #[test]
    fn source_limit_and_untrusted_authority_have_no_committed_reference() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        let mut db = Database::open(&root).unwrap();
        let req = request(&db);
        assert_eq!(
            db.register_asset(
                RequestContext::unverified_external(),
                &req,
                &mut Cursor::new(b"secret"),
                1024
            )
            .unwrap_err()
            .code,
            ErrorCode::PermissionDenied
        );
        assert_eq!(
            db.register_asset(
                RequestContext::local_ui(),
                &req,
                &mut Cursor::new(b"oversized"),
                2
            )
            .unwrap_err()
            .code,
            ErrorCode::InvalidRequest
        );
        assert_eq!(
            db.connection
                .query_row("SELECT count(*) FROM assets", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
    #[test]
    fn missing_registered_asset_preserves_reference_and_blocks_new_mutations() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        let mut db = Database::open(&root).unwrap();
        let req = request(&db);
        let info = db
            .register_asset(
                RequestContext::local_ui(),
                &req,
                &mut Cursor::new(b"original"),
                1024,
            )
            .unwrap();
        std::fs::remove_file(asset_path(&root, &info.digest)).unwrap();
        drop(db);
        let mut reopened = Database::open(&root).unwrap();
        assert_eq!(reopened.recovery_report().missing_assets, 1);
        assert_eq!(
            reopened
                .connection
                .query_row("SELECT count(*) FROM assets", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            1
        );
        let save = SavePreferences {
            contract_version: 1,
            workspace_id: reopened.id.clone(),
            operation_id: OperationId::generate(),
            expected_revision: 0,
            preferences: ReadingPreferences::default(),
        };
        assert_eq!(
            reopened
                .save_preferences(RequestContext::local_ui(), &save)
                .unwrap_err()
                .code,
            ErrorCode::RecoveryRequired
        );
    }

    #[test]
    fn publication_failures_cannot_commit_a_reference_to_incomplete_bytes() {
        for phase in [
            Phase::Write,
            Phase::FileSync,
            Phase::Publish,
            Phase::DirectorySync,
            Phase::BeforeCommit,
        ] {
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path().join("workspace");
            let mut db = Database::open(&root).unwrap();
            let req = request(&db);
            let result = faults::using(
                move |point| {
                    if point == phase {
                        Err(AppError::new(ErrorCode::StorageFull))
                    } else {
                        Ok(())
                    }
                },
                || {
                    db.register_asset(
                        RequestContext::local_ui(),
                        &req,
                        &mut Cursor::new(b"original"),
                        1024,
                    )
                },
            );
            assert!(result.is_err(), "{phase:?}");
            assert_eq!(
                db.connection
                    .query_row("SELECT count(*) FROM assets", [], |r| r.get::<_, i64>(0))
                    .unwrap(),
                0,
                "{phase:?}"
            );
            drop(db);
            let reopened = Database::open(&root).unwrap();
            assert_eq!(
                reopened
                    .connection
                    .query_row("SELECT count(*) FROM assets", [], |r| r.get::<_, i64>(0))
                    .unwrap(),
                0
            );
        }
    }

    #[test]
    fn lost_reply_after_asset_commit_can_only_replay_the_original_receipt() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        let mut db = Database::open(&root).unwrap();
        let req = request(&db);
        let result = faults::using(
            |point| {
                if point == Phase::AfterCommit {
                    Err(AppError::new(ErrorCode::StorageFailure))
                } else {
                    Ok(())
                }
            },
            || {
                db.register_asset(
                    RequestContext::local_ui(),
                    &req,
                    &mut Cursor::new(b"original"),
                    1024,
                )
            },
        );
        assert_eq!(result.unwrap_err().code, ErrorCode::OutcomeUnconfirmed);
        drop(db);
        let mut reopened = Database::open(&root).unwrap();
        let info = reopened
            .register_asset(
                RequestContext::local_ui(),
                &req,
                &mut Cursor::new(b"original"),
                1024,
            )
            .unwrap();
        assert_eq!(info.byte_length, 8);
        assert_eq!(
            reopened
                .connection
                .query_row("SELECT count(*) FROM assets", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(
            reopened
                .connection
                .query_row("SELECT count(*) FROM operations", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(
            reopened
                .register_asset(
                    RequestContext::local_ui(),
                    &req,
                    &mut Cursor::new(b"different"),
                    1024
                )
                .unwrap_err()
                .code,
            ErrorCode::IdempotencyConflict
        );
    }

    #[test]
    fn existing_corrupt_bytes_are_never_replaced_by_deduplication() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        let mut db = Database::open(&root).unwrap();
        let req = request(&db);
        let info = db
            .register_asset(
                RequestContext::local_ui(),
                &req,
                &mut Cursor::new(b"original"),
                1024,
            )
            .unwrap();
        let path = asset_path(&root, &info.digest);
        std::fs::write(&path, b"damage!!").unwrap();
        assert_eq!(
            db.register_asset(
                RequestContext::local_ui(),
                &req,
                &mut Cursor::new(b"original"),
                1024
            )
            .unwrap_err()
            .code,
            ErrorCode::RecoveryRequired
        );
        assert_eq!(std::fs::read(&path).unwrap(), b"damage!!");
    }

    #[test]
    fn startup_detects_same_length_corruption_and_does_not_follow_shard_links() {
        use std::os::unix::fs::symlink;
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        let mut db = Database::open(&root).unwrap();
        let req = request(&db);
        let info = db
            .register_asset(
                RequestContext::local_ui(),
                &req,
                &mut Cursor::new(b"original"),
                1024,
            )
            .unwrap();
        let path = asset_path(&root, &info.digest);
        std::fs::write(&path, b"damage!!").unwrap();
        drop(db);
        let reopened = Database::open(&root).unwrap();
        assert_eq!(reopened.recovery_report().corrupt_assets, 1);
        drop(reopened);
        let shard = path.parent().unwrap();
        let outside = temp.path().join("outside");
        std::fs::rename(shard, &outside).unwrap();
        symlink(&outside, shard).unwrap();
        let mut reopened = Database::open(&root).unwrap();
        assert_eq!(reopened.recovery_report().unsafe_paths, 1);
        assert_eq!(
            reopened
                .register_asset(
                    RequestContext::local_ui(),
                    &req,
                    &mut Cursor::new(b"original"),
                    1024
                )
                .unwrap_err()
                .code,
            ErrorCode::RecoveryRequired
        );
        assert_eq!(
            std::fs::read(outside.join(path.file_name().unwrap())).unwrap(),
            b"damage!!"
        );
    }

    #[test]
    #[ignore = "private child process launched by killed_asset_writer_reconciles_without_replaying"]
    fn crash_child() {
        let root = std::path::PathBuf::from(std::env::var_os("MEMOQUILL_TEST_ASSET_ROOT").unwrap());
        let mut db = Database::open(&root).unwrap();
        let operation_id =
            OperationId::try_from(std::env::var("MEMOQUILL_TEST_ASSET_OPERATION").unwrap())
                .unwrap();
        let req = RegisterAsset {
            contract_version: 1,
            workspace_id: db.id.clone(),
            operation_id,
        };
        let phase = match std::env::var("MEMOQUILL_TEST_ASSET_PHASE")
            .unwrap()
            .as_str()
        {
            "directory" => Phase::DirectorySync,
            "before" => Phase::BeforeCommit,
            "after" => Phase::AfterCommit,
            _ => panic!("unknown test phase"),
        };
        faults::using(
            move |point| {
                if point == phase {
                    std::fs::write(root.join("asset-child-ready"), b"ready").unwrap();
                    std::thread::sleep(std::time::Duration::from_secs(15));
                }
                Ok(())
            },
            || {
                db.register_asset(
                    RequestContext::local_ui(),
                    &req,
                    &mut Cursor::new(b"crash original"),
                    1024,
                )
            },
        )
        .unwrap();
    }

    #[test]
    fn killed_asset_writer_reconciles_without_replaying() {
        use std::{
            process::{Command, Stdio},
            time::{Duration, Instant},
        };
        for phase in ["directory", "before", "after"] {
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path().join("workspace");
            let db = Database::open(&root).unwrap();
            let req = request(&db);
            drop(db);
            let mut child = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--ignored",
                    "--exact",
                    "workspace::assets::tests::crash_child",
                ])
                .env("MEMOQUILL_TEST_ASSET_ROOT", &root)
                .env("MEMOQUILL_TEST_ASSET_OPERATION", req.operation_id.as_str())
                .env("MEMOQUILL_TEST_ASSET_PHASE", phase)
                .stdout(Stdio::null())
                .spawn()
                .unwrap();
            let start = Instant::now();
            while !root.join("asset-child-ready").exists()
                && start.elapsed() < Duration::from_secs(5)
            {
                std::thread::sleep(Duration::from_millis(10));
            }
            let ready = root.join("asset-child-ready").exists();
            let _ = child.kill();
            child.wait().unwrap();
            assert!(ready, "{phase}");
            let mut reopened = Database::open(&root).unwrap();
            let receipt = reopened
                .operation_receipt(RequestContext::local_ui(), &req.operation_id)
                .unwrap()
                .unwrap();
            let count = reopened
                .connection
                .query_row("SELECT count(*) FROM assets", [], |r| r.get::<_, i64>(0))
                .unwrap();
            if phase == "after" {
                assert_eq!(count, 1);
                assert!(matches!(receipt.result, ReceiptResult::Applied(_)));
                assert_eq!(
                    reopened
                        .register_asset(
                            RequestContext::local_ui(),
                            &req,
                            &mut Cursor::new(b"crash original"),
                            1024
                        )
                        .unwrap()
                        .byte_length,
                    14
                );
                assert_eq!(reopened.recovery_report().orphan_assets, 0);
            } else {
                assert_eq!(count, 0);
                assert_eq!(
                    receipt.result,
                    ReceiptResult::Interrupted(ErrorCode::RecoveryRequired)
                );
                assert_eq!(reopened.recovery_report().interrupted_operations, 1);
                assert_eq!(reopened.recovery_report().orphan_assets, 1);
            }
            assert_eq!(
                reopened
                    .connection
                    .query_row("SELECT count(*) FROM operations", [], |r| r
                        .get::<_, i64>(0))
                    .unwrap(),
                1
            );
        }
    }
    #[test]
    #[ignore = "private FIFO probe launched by special_assets_do_not_block_recovery"]
    fn fifo_child() {
        let root = PathBuf::from(std::env::var_os("MEMOQUILL_TEST_FIFO_ROOT").unwrap());
        let mut db = Database::open(&root).unwrap();
        let req = request(&db);
        let info = db
            .register_asset(RequestContext::local_ui(), &req, &mut Cursor::new(b""), 1)
            .unwrap();
        let path = asset_path(&root, &info.digest);
        std::fs::remove_file(&path).unwrap();
        rustix::fs::mkfifoat(
            rustix::fs::CWD,
            &path,
            rustix::fs::Mode::RUSR | rustix::fs::Mode::WUSR,
        )
        .unwrap();
        assert_eq!(
            verify(&root, &info).unwrap_err().code,
            ErrorCode::RecoveryRequired
        );
        drop(db);
        let reopened = Database::open(&root).unwrap();
        assert_eq!(reopened.recovery_report().corrupt_assets, 1);
        assert_eq!(reopened.recovery_report().unsafe_paths, 1);
        drop(reopened);
        let database = root.join("memoquill.sqlite3");
        std::fs::remove_file(&database).unwrap();
        rustix::fs::mkfifoat(
            rustix::fs::CWD,
            &database,
            rustix::fs::Mode::RUSR | rustix::fs::Mode::WUSR,
        )
        .unwrap();
        assert_eq!(
            Database::open(&root).err().unwrap().code,
            ErrorCode::RecoveryRequired
        );
        std::fs::write(root.join("fifo-rejected"), b"done").unwrap();
    }
    #[test]
    fn special_assets_do_not_block_recovery() {
        use std::{
            process::{Command, Stdio},
            time::{Duration, Instant},
        };
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--ignored",
                "--exact",
                "workspace::assets::tests::fifo_child",
            ])
            .env("MEMOQUILL_TEST_FIFO_ROOT", &root)
            .stdout(Stdio::null())
            .spawn()
            .unwrap();
        let start = Instant::now();
        let mut status = None;
        while start.elapsed() < Duration::from_secs(2) {
            status = child.try_wait().unwrap();
            if status.is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        if status.is_none() {
            let _ = child.kill();
        }
        let exit = child.wait().unwrap();
        assert!(
            status.is_some(),
            "Recovery blocked on a FIFO instead of rejecting it"
        );
        assert!(exit.success());
        assert!(root.join("fifo-rejected").is_file());
    }
    #[test]
    fn invalid_asset_contracts_and_limits_do_not_admit_or_stage_sources() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        let mut db = Database::open(&root).unwrap();
        let mut req = request(&db);
        for (version, limit) in [(2, 1024), (1, 0), (1, i64::MAX as u64 + 1)] {
            req.contract_version = version;
            assert_eq!(
                db.register_asset(
                    RequestContext::local_ui(),
                    &req,
                    &mut Cursor::new(b"x"),
                    limit
                )
                .unwrap_err()
                .code,
                ErrorCode::InvalidRequest
            );
        }
        assert_eq!(std::fs::read_dir(root.join("staging")).unwrap().count(), 0);
        assert_eq!(
            db.connection
                .query_row("SELECT count(*) FROM operations", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
}
