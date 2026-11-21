//! Preserve evidence after a crash. Never delete, replay, or repair source bytes.
use super::{assets, error::Result, owner, *};
use rusqlite::{Connection, TransactionBehavior};
use std::{collections::BTreeSet, fs, path::Path, thread};

pub(super) fn reconcile(connection: &mut Connection, root: &Path) -> Result<RecoveryReport> {
    let interrupted_operations = interrupt_prepared(connection)?;
    let mut report = RecoveryReport {
        interrupted_operations,
        ..RecoveryReport::default()
    };
    let registered = registered_assets(connection)?;
    let workers = thread::available_parallelism()
        .map_or(1, |count| count.get())
        .min(4)
        .min(registered.len() / 64)
        .max(1);
    let (missing, corrupt) = inspect_registered_assets(root, &registered, workers)?;
    report.missing_assets = missing;
    report.corrupt_assets = corrupt;
    let known = registered
        .iter()
        .map(|info| info.digest.as_str().to_owned())
        .collect();
    inspect_asset_tree(root, &known, &mut report)?;
    inspect_staging(root, &mut report)?;
    Ok(report)
}

fn inspect_registered_assets(
    root: &Path,
    assets: &[AssetInfo],
    workers: usize,
) -> Result<(u64, u64)> {
    fn inspect(root: &Path, assets: &[AssetInfo]) -> Result<(u64, u64)> {
        let mut report = RecoveryReport::default();
        for info in assets {
            inspect_registered(root, info, &mut report)?;
        }
        Ok((report.missing_assets, report.corrupt_assets))
    }
    if workers == 1 || assets.is_empty() {
        return inspect(root, assets);
    }
    // Workers verify immutable files only. The workspace owner keeps the lock,
    // and no database access or decision to enable writes leaves this thread.
    thread::scope(|scope| {
        let checks = assets
            .chunks(assets.len().div_ceil(workers))
            .map(|chunk| thread::Builder::new().spawn_scoped(scope, move || inspect(root, chunk)))
            .collect::<std::io::Result<Vec<_>>>()?;
        checks
            .into_iter()
            .try_fold((0, 0), |(missing, corrupt), check| {
                let (chunk_missing, chunk_corrupt) = check
                    .join()
                    .map_err(|_| AppError::new(ErrorCode::StorageFailure))??;
                Ok((missing + chunk_missing, corrupt + chunk_corrupt))
            })
    })
}
fn inspect_staging(root: &Path, report: &mut RecoveryReport) -> Result<()> {
    for entry in fs::read_dir(root.join("staging"))? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            report.staging_files += 1;
        } else {
            report.unsafe_paths += 1;
        }
    }
    Ok(())
}

fn interrupt_prepared(connection: &mut Connection) -> Result<u64> {
    let ids = connection
        .prepare("SELECT id FROM operations WHERE state='prepared'")?
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    for id in &ids {
        let id = OperationId::try_from(id.clone())
            .map_err(|_| AppError::new(ErrorCode::RecoveryRequired))?;
        assets::interrupt(&tx, &id, AppError::new(ErrorCode::RecoveryRequired))?;
    }
    tx.commit()?;
    Ok(ids.len() as u64)
}

fn registered_assets(connection: &Connection) -> Result<Vec<AssetInfo>> {
    let rows = connection
        .prepare("SELECT digest,byte_length FROM assets")?
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    rows.into_iter()
        .map(|(digest, length)| {
            let digest = AssetDigest::try_from(digest)
                .map_err(|_| AppError::new(ErrorCode::RecoveryRequired))?;
            let byte_length =
                u64::try_from(length).map_err(|_| AppError::new(ErrorCode::RecoveryRequired))?;
            Ok(AssetInfo {
                digest,
                byte_length,
            })
        })
        .collect()
}

fn inspect_registered(root: &Path, info: &AssetInfo, report: &mut RecoveryReport) -> Result<()> {
    let path = assets::asset_path(root, &info.digest);
    // Inspect the shard before touching a child: NOFOLLOW on a leaf alone is insufficient.
    let parent = path
        .parent()
        .ok_or(AppError::new(ErrorCode::RecoveryRequired))?;
    if fs::symlink_metadata(parent).is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound) {
        report.missing_assets += 1;
        return Ok(());
    }
    if owner::private_directory(parent).is_err() {
        report.corrupt_assets += 1;
        return Ok(());
    }
    match fs::symlink_metadata(&path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => report.missing_assets += 1,
        Err(error) => return Err(error.into()),
        Ok(_) if assets::verify(root, info).is_err() => report.corrupt_assets += 1,
        Ok(_) => {}
    }
    Ok(())
}

fn inspect_asset_tree(
    root: &Path,
    known: &BTreeSet<String>,
    report: &mut RecoveryReport,
) -> Result<()> {
    for entry in fs::read_dir(root.join("assets/sha256"))? {
        let entry = entry?;
        let name = entry.file_name();
        let Some(shard) = name.to_str().filter(|s| hex_name(s, 2)) else {
            report.unsafe_paths += 1;
            continue;
        };
        if owner::private_directory(&entry.path()).is_err() {
            report.unsafe_paths += 1;
            continue;
        }
        inspect_shard(&entry.path(), shard, known, report)?;
    }
    Ok(())
}

fn inspect_shard(
    path: &Path,
    shard: &str,
    known: &BTreeSet<String>,
    report: &mut RecoveryReport,
) -> Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let name = entry.file_name();
        let Some(tail) = name.to_str().filter(|s| hex_name(s, 62)) else {
            report.unsafe_paths += 1;
            continue;
        };
        if !entry.file_type()?.is_file() {
            report.unsafe_paths += 1;
        } else if !known.contains(&format!("{shard}{tail}")) {
            report.orphan_assets += 1;
        }
    }
    Ok(())
}

fn hex_name(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}

#[cfg(test)]
mod tests {
    use super::super::database::Database;
    use super::*;
    use std::{io::Cursor, os::unix::fs::symlink};

    #[test]
    fn parallel_recovery_reports_corruption_and_missing_assets() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        let mut db = Database::open(&root).unwrap();
        let mut registered = Vec::new();
        for original in [b"valid", b"alter", b"absent".as_slice()] {
            let request = RegisterAsset {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
            };
            registered.push(
                db.register_asset(
                    RequestContext::local_ui(),
                    &request,
                    &mut Cursor::new(original),
                    1024,
                )
                .unwrap(),
            );
        }
        fs::write(assets::asset_path(&root, &registered[1].digest), b"other").unwrap();
        fs::remove_file(assets::asset_path(&root, &registered[2].digest)).unwrap();
        assert_eq!(
            inspect_registered_assets(&root, &registered, 2).unwrap(),
            (1, 1)
        );
    }

    #[test]
    #[ignore = "Explicit paired release measurement on the existing synthetic SSD fixture"]
    fn compare_scale_recovery_hashing() {
        let repository = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let source = repository.join(".qualification/m5-scale/full-v1/workspace");
        let database = source.join("memoquill.sqlite3");
        assert!(database.is_file());
        let connection =
            Connection::open_with_flags(database, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
                .unwrap();
        let registered = registered_assets(&connection).unwrap();
        assert_eq!(registered.len(), 10_000);
        assert_eq!(
            registered
                .iter()
                .map(|asset| asset.byte_length)
                .sum::<u64>(),
            10_000_000_000
        );
        let mut samples = Vec::new();
        let parallel = thread::available_parallelism().unwrap().get().min(4);
        assert!(parallel > 1);
        for workers in [1, parallel, parallel, 1] {
            let start = std::time::Instant::now();
            assert_eq!(
                inspect_registered_assets(&source, &registered, workers).unwrap(),
                (0, 0)
            );
            samples.push(serde_json::json!({
                "workers": workers,
                "seconds": start.elapsed().as_secs_f64(),
            }));
        }
        let output = repository.join(".qualification/m5-optimized-scale/recovery-hashing.json");
        fs::write(output, serde_json::to_vec_pretty(&samples).unwrap()).unwrap();
    }
    #[test]
    fn audit_reports_missing_shards_staging_orphans_and_unsafe_entries_without_removing_them() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        let mut db = Database::open(&root).unwrap();
        let request = RegisterAsset {
            contract_version: 1,
            workspace_id: db.id.clone(),
            operation_id: OperationId::generate(),
        };
        let info = db
            .register_asset(
                RequestContext::local_ui(),
                &request,
                &mut Cursor::new(b"original"),
                1024,
            )
            .unwrap();
        let path = assets::asset_path(&root, &info.digest);
        fs::remove_file(&path).unwrap();
        fs::remove_dir(path.parent().unwrap()).unwrap();
        let outside = temp.path().join("outside");
        fs::write(&outside, b"preserve").unwrap();
        fs::write(root.join("staging/leftover.part"), b"partial").unwrap();
        symlink(&outside, root.join("staging/alias")).unwrap();
        let assets = root.join("assets/sha256");
        owner::create_private_tree(&assets.join("gg")).unwrap();
        fs::write(assets.join("gg").join("a".repeat(62)), b"untrusted").unwrap();
        fs::write(assets.join("unexpected"), b"untrusted").unwrap();
        owner::create_private_tree(&assets.join("aa")).unwrap();
        fs::write(assets.join("aa/bad-tail"), b"untrusted").unwrap();
        fs::write(assets.join("aa").join("c".repeat(62)), b"orphan").unwrap();
        symlink(&outside, assets.join("aa").join("b".repeat(62))).unwrap();
        drop(db);
        let reopened = Database::open(&root).unwrap();
        let report = reopened.recovery_report();
        assert_eq!(report.missing_assets, 1);
        assert_eq!(report.corrupt_assets, 0);
        assert_eq!(report.staging_files, 1);
        assert_eq!(report.orphan_assets, 1);
        assert_eq!(report.unsafe_paths, 5);
        assert_eq!(fs::read(outside).unwrap(), b"preserve");
        assert_eq!(
            fs::read(root.join("staging/leftover.part")).unwrap(),
            b"partial"
        );
    }
}
