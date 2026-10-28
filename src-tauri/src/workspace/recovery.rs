//! Preserve evidence after a crash. Never delete, replay, or repair source bytes.
use super::{assets, error::Result, owner, *};
use rusqlite::{Connection, TransactionBehavior};
use std::{collections::BTreeSet, fs, path::Path};

pub(super) fn reconcile(connection: &mut Connection, root: &Path) -> Result<RecoveryReport> {
    let interrupted_operations = interrupt_prepared(connection)?;
    let mut report = RecoveryReport {
        interrupted_operations,
        ..RecoveryReport::default()
    };
    let registered = registered_assets(connection)?;
    for info in &registered {
        inspect_registered(root, info, &mut report)?;
    }
    let known = registered
        .iter()
        .map(|info| info.digest.as_str().to_owned())
        .collect();
    inspect_asset_tree(root, &known, &mut report)?;
    inspect_staging(root, &mut report)?;
    Ok(report)
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
