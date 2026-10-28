use super::{
    AppError, ErrorCode, PreferencesSnapshot, ReadingPreferences, WorkspaceId,
    error::Result,
    migrations,
    owner::{self, Owner},
};
use rusqlite::{
    Connection, OpenFlags, TransactionBehavior,
    backup::{Backup, StepResult},
    params,
};
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

pub(super) const APPLICATION_ID: i32 = 0x4d514c4c;
pub(super) const SCHEMA_VERSION: i64 = 2;
pub(super) const BUSY_MS: u64 = 2000;

pub(super) struct Database {
    pub(super) connection: Connection,
    pub(super) owner: Owner,
    pub(super) id: WorkspaceId,
    pub(super) recovery: super::RecoveryReport,
}
impl Database {
    pub(super) fn recovery_report(&self) -> super::RecoveryReport {
        self.recovery.clone()
    }
    pub(super) fn open(root: &Path) -> Result<Self> {
        let owner = Owner::acquire(root)?;
        runtime_policy(rusqlite::version_number())?;
        initialise_if_missing(root)?;
        let (connection, id) = open_current(root)?;
        let mut db = Self {
            connection,
            owner,
            id,
            recovery: super::RecoveryReport::default(),
        };
        db.preferences()?;
        db.recovery = super::recovery::reconcile(&mut db.connection, root)?;
        Ok(db)
    }
    pub(super) fn preferences(&self) -> Result<PreferencesSnapshot> {
        let (revision, data) = self.connection.query_row(
            "SELECT revision,data FROM preferences WHERE workspace_id=?1",
            [self.id.as_str()],
            |r| Ok((r.get(0)?, r.get::<_, String>(1)?)),
        )?;
        let preferences =
            serde_json::from_str(&data).map_err(|_| AppError::new(ErrorCode::RecoveryRequired))?;
        if !(0..=super::types::MAX_REVISION).contains(&revision) {
            return Err(AppError::new(ErrorCode::RecoveryRequired));
        }
        Ok(PreferencesSnapshot {
            workspace_id: self.id.clone(),
            revision,
            preferences,
        })
    }
}

fn initialise_if_missing(root: &Path) -> Result<()> {
    let path = root.join("memoquill.sqlite3");
    match std::fs::symlink_metadata(&path) {
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            publish_initial_database(root, &path)
        }
        Err(error) => Err(error.into()),
    }
}
fn publish_initial_database(root: &Path, path: &Path) -> Result<()> {
    prepare_directories(root)?;
    let staged = root
        .join("staging")
        .join(format!("init-{}.sqlite3", WorkspaceId::generate().as_str()));
    bootstrap(&staged, SCHEMA_VERSION)?;
    owner::publish_no_replace(&staged, path)?;
    owner::sync_directory(&root.join("staging"))?;
    owner::sync_directory(root)
}
fn open_current(root: &Path) -> Result<(Connection, WorkspaceId)> {
    let path = root.join("memoquill.sqlite3");
    let (from, id) = preflight(&path)?;
    prepare_directories(root)?;
    let mut connection = open_connection(&path, Access::ReadWrite)?;
    runtime_configuration(&connection)?;
    if from < SCHEMA_VERSION {
        snapshot(&connection, root, from)?;
        migrations::apply(&mut connection, from, migrations::MIGRATIONS)?;
    }
    Ok((connection, id))
}

#[derive(Clone, Copy)]
pub(super) enum Access {
    ReadOnly,
    ReadWrite,
}
pub(super) fn open_connection(path: &Path, access: Access) -> Result<Connection> {
    let access = match access {
        Access::ReadOnly => OpenFlags::SQLITE_OPEN_READ_ONLY,
        Access::ReadWrite => OpenFlags::SQLITE_OPEN_READ_WRITE,
    };
    let connection = Connection::open_with_flags(
        path,
        access | OpenFlags::SQLITE_OPEN_NO_MUTEX | OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )?;
    base_policy(&connection)?;
    Ok(connection)
}

fn runtime_policy(version: i32) -> Result<()> {
    if version < 3_051_003 {
        return Err(AppError::new(ErrorCode::UnsupportedSchema));
    }
    Ok(())
}
fn base_policy(connection: &Connection) -> Result<()> {
    use rusqlite::config::DbConfig;
    connection.set_db_config(DbConfig::SQLITE_DBCONFIG_DEFENSIVE, true)?;
    connection.set_db_config(DbConfig::SQLITE_DBCONFIG_DQS_DML, false)?;
    connection.set_db_config(DbConfig::SQLITE_DBCONFIG_DQS_DDL, false)?;
    connection.busy_timeout(Duration::from_millis(BUSY_MS))?;
    connection.pragma_update(None, "foreign_keys", true)?;
    connection.pragma_update(None, "trusted_schema", false)?;
    Ok(())
}
fn runtime_configuration(connection: &Connection) -> Result<()> {
    let mode: String = connection.query_row("PRAGMA journal_mode=WAL", [], |r| r.get(0))?;
    connection.pragma_update(None, "synchronous", "FULL")?;
    let fk: i64 = connection.query_row("PRAGMA foreign_keys", [], |r| r.get(0))?;
    if mode != "wal" || fk != 1 {
        return Err(AppError::new(ErrorCode::StorageFailure));
    }
    Ok(())
}
fn prepare_directories(root: &Path) -> Result<()> {
    for name in ["staging", "migration-backups", "assets", "assets/sha256"] {
        let path = root.join(name);
        owner::create_private_tree(&path)?;
        owner::private_directory(&path)?;
    }
    Ok(())
}
fn preflight(path: &Path) -> Result<(i64, WorkspaceId)> {
    validate_database_file(path)?;
    let connection = open_connection(path, Access::ReadOnly)?;
    let version = validated_schema(&connection)?;
    migrations::validate(&connection, version)
        .map_err(|_| AppError::new(ErrorCode::RecoveryRequired))?;
    let id: String =
        connection.query_row("SELECT id FROM workspace_meta WHERE singleton=1", [], |r| {
            r.get(0)
        })?;
    let id = WorkspaceId::try_from(id).map_err(|_| AppError::new(ErrorCode::RecoveryRequired))?;
    Ok((version, id))
}
fn validate_database_file(path: &Path) -> Result<()> {
    if !std::fs::symlink_metadata(path)?.is_file() {
        return Err(AppError::new(ErrorCode::RecoveryRequired));
    }
    Ok(())
}
fn validated_schema(connection: &Connection) -> Result<i64> {
    let app: i32 = connection.query_row("PRAGMA application_id", [], |r| r.get(0))?;
    let version: i64 = connection.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    if app != APPLICATION_ID || !(1..=SCHEMA_VERSION).contains(&version) {
        return Err(AppError::new(ErrorCode::UnsupportedSchema));
    }
    Ok(version)
}

pub(super) fn bootstrap(path: &Path, through: i64) -> Result<()> {
    drop(owner::create_private_file(path)?);
    let mut connection = open_connection(path, Access::ReadWrite)?;
    connection.pragma_update(None, "journal_mode", "DELETE")?;
    connection.pragma_update(None, "synchronous", "EXTRA")?;
    migrations::apply(
        &mut connection,
        0,
        &migrations::MIGRATIONS[..through as usize],
    )?;
    seed_workspace(&mut connection)?;
    close_and_sync(connection, path)
}
fn seed_workspace(connection: &mut Connection) -> Result<()> {
    let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let id = WorkspaceId::generate();
    tx.execute(
        "INSERT INTO workspace_meta(singleton,id) VALUES (1,?1)",
        [id.as_str()],
    )?;
    tx.execute(
        "INSERT INTO preferences(workspace_id,revision,data) VALUES (?1,0,?2)",
        params![
            id.as_str(),
            serde_json::to_string(&ReadingPreferences::default())
                .map_err(|_| AppError::new(ErrorCode::InvalidRequest))?
        ],
    )?;
    tx.pragma_update(None, "application_id", APPLICATION_ID)?;
    tx.commit()?;
    Ok(())
}

pub(super) fn snapshot(source: &Connection, root: &Path, from: i64) -> Result<PathBuf> {
    let folder = root.join("migration-backups");
    let path = folder.join(format!(
        "before-{from}-{}.sqlite3",
        WorkspaceId::generate().as_str()
    ));
    drop(owner::create_private_file(&path)?);
    let mut destination = open_connection(&path, Access::ReadWrite)?;
    destination.pragma_update(None, "synchronous", "EXTRA")?;
    bounded_backup(source, &mut destination)?;
    destination.pragma_update(None, "journal_mode", "DELETE")?;
    integrity(&destination)?;
    close_and_sync(destination, &path)?;
    owner::sync_directory(&folder)?;
    Ok(path)
}
fn bounded_backup(source: &Connection, destination: &mut Connection) -> Result<()> {
    let backup = Backup::new(source, destination)?;
    let start = Instant::now();
    loop {
        if backup_expired(start.elapsed()) {
            return Err(AppError::new(ErrorCode::Busy));
        }
        match backup.step(128)? {
            StepResult::Done => return Ok(()),
            StepResult::More => {}
            _ => std::thread::sleep(Duration::from_millis(10)),
        }
    }
}
fn backup_expired(elapsed: Duration) -> bool {
    elapsed >= Duration::from_secs(30)
}
pub(super) fn integrity(connection: &Connection) -> Result<()> {
    let value: String = connection.query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
    let mut fk = connection.prepare("PRAGMA foreign_key_check")?;
    if value != "ok" || fk.query([])?.next()?.is_some() {
        return Err(AppError::new(ErrorCode::RecoveryRequired));
    }
    Ok(())
}
fn close_and_sync(connection: Connection, path: &Path) -> Result<()> {
    connection
        .close()
        .map_err(|(_, error)| AppError::from(error))?;
    std::fs::File::open(path)?.sync_all()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::ErrorCode;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn all_connection_roles_enforce_defensive_sql_and_disable_extension_loading() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        let db = Database::open(&root).unwrap();
        let readonly = open_connection(&root.join("memoquill.sqlite3"), Access::ReadOnly).unwrap();
        for connection in [&db.connection, &readonly] {
            assert!(
                connection
                    .db_config(rusqlite::config::DbConfig::SQLITE_DBCONFIG_DEFENSIVE)
                    .unwrap()
            );
            assert!(
                connection
                    .query_row("SELECT \"unknown_literal\"", [], |r| r.get::<_, String>(0))
                    .is_err()
            );
            let error = connection
                .query_row(
                    "SELECT load_extension('memoquill-must-never-load')",
                    [],
                    |r| r.get::<_, String>(0),
                )
                .unwrap_err();
            assert!(error.to_string().contains("not authorized"));
        }
    }

    #[test]
    fn new_workspace_is_durable_identifiable_and_connection_policy_is_verified() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        let db = Database::open(&root).unwrap();
        let id = db.id.clone();
        assert_eq!(
            db.connection
                .query_row("PRAGMA foreign_keys", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(
            db.connection
                .query_row("PRAGMA journal_mode", [], |r| r.get::<_, String>(0))
                .unwrap(),
            "wal"
        );
        assert_eq!(
            db.connection
                .query_row("PRAGMA synchronous", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            2
        );
        assert_eq!(
            db.connection
                .query_row("PRAGMA busy_timeout", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            2000
        );
        assert_eq!(
            db.connection
                .query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            SCHEMA_VERSION
        );
        assert!(rusqlite::version_number() >= 3_051_003);
        assert_eq!(
            std::fs::metadata(root.join("memoquill.sqlite3"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        drop(db);
        assert_eq!(
            Database::open(&root)
                .unwrap()
                .preferences()
                .unwrap()
                .workspace_id,
            id
        );
    }

    #[test]
    fn newer_or_foreign_schema_is_not_initialised_or_overwritten() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        drop(Owner::acquire(&root).unwrap());
        let path = root.join("memoquill.sqlite3");
        let db = Connection::open(&path).unwrap();
        db.pragma_update(None, "application_id", APPLICATION_ID)
            .unwrap();
        db.pragma_update(None, "user_version", 999).unwrap();
        db.close().unwrap();
        let before = std::fs::read(&path).unwrap();
        assert_eq!(
            Database::open(&root).err().unwrap().code,
            ErrorCode::UnsupportedSchema
        );
        assert_eq!(std::fs::read(path).unwrap(), before);
    }

    #[test]
    fn unknown_bytes_are_preserved_for_recovery() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        drop(Owner::acquire(&root).unwrap());
        let path = root.join("memoquill.sqlite3");
        std::fs::write(&path, b"not a Memoquill workspace").unwrap();
        assert!(Database::open(&root).is_err());
        assert_eq!(std::fs::read(path).unwrap(), b"not a Memoquill workspace");
    }
    fn legacy_root() -> (tempfile::TempDir, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        let owner = Owner::acquire(&root).unwrap();
        prepare_directories(&root).unwrap();
        let path = root.join("memoquill.sqlite3");
        bootstrap(&path, 1).unwrap();
        let connection = open_connection(&path, Access::ReadWrite).unwrap();
        connection
            .execute(
                "UPDATE preferences SET revision=7,data=?1",
                [r#"{"theme":"dark","textScale":125}"#],
            )
            .unwrap();
        connection.close().unwrap();
        drop(owner);
        (temp, root)
    }

    #[test]
    fn upgrade_keeps_values_and_a_verified_standalone_snapshot() {
        let (_temp, root) = legacy_root();
        let db = Database::open(&root).unwrap();
        assert_eq!(db.preferences().unwrap().revision, 7);
        assert_eq!(db.preferences().unwrap().preferences.text_scale.get(), 125);
        let files = std::fs::read_dir(root.join("migration-backups"))
            .unwrap()
            .collect::<std::io::Result<Vec<_>>>()
            .unwrap();
        assert_eq!(files.len(), 1);
        let copy = open_connection(&files[0].path(), Access::ReadOnly).unwrap();
        integrity(&copy).unwrap();
        assert_eq!(
            copy.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(
            copy.query_row("SELECT revision FROM preferences", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            7
        );
        assert_eq!(
            copy.query_row("PRAGMA foreign_keys", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            1
        );
    }

    #[test]
    fn broken_migration_rolls_back_ddl_data_and_version() {
        let (_temp, root) = legacy_root();
        let mut connection =
            open_connection(&root.join("memoquill.sqlite3"), Access::ReadWrite).unwrap();
        let broken = [migrations::Migration {
            version: 2,
            sql: "CREATE TABLE should_rollback(id INTEGER); UPDATE preferences SET revision=99; THIS IS INVALID SQL;",
        }];
        assert!(migrations::apply(&mut connection, 1, &broken).is_err());
        assert_eq!(
            connection
                .query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(
            connection
                .query_row("SELECT revision FROM preferences", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            7
        );
        assert_eq!(
            connection
                .query_row(
                    "SELECT count(*) FROM sqlite_schema WHERE name='should_rollback'",
                    [],
                    |r| r.get::<_, i64>(0)
                )
                .unwrap(),
            0
        );
        integrity(&connection).unwrap();
    }

    #[test]
    fn changed_migration_checksum_is_not_repaired_by_overwriting_it() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        let db = Database::open(&root).unwrap();
        db.connection
            .execute(
                "UPDATE schema_migrations SET checksum='changed' WHERE version=1",
                [],
            )
            .unwrap();
        drop(db);
        let path = root.join("memoquill.sqlite3");
        let before = std::fs::read(&path).unwrap();
        assert_eq!(
            Database::open(&root).err().unwrap().code,
            ErrorCode::RecoveryRequired
        );
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }

    #[test]
    fn real_full_and_busy_failures_leave_prior_data_usable() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        let mut db = Database::open(&root).unwrap();
        let pages: i64 = db
            .connection
            .query_row("PRAGMA page_count", [], |r| r.get(0))
            .unwrap();
        db.connection
            .pragma_update(None, "max_page_count", pages)
            .unwrap();
        let tx = db.connection.transaction().unwrap();
        let huge = serde_json::to_string(&"x".repeat(1_000_000)).unwrap();
        let error=tx.execute("INSERT INTO operations(id,workspace_id,principal,request_hash,kind,state,result) VALUES ('full',?1,'local-ui','hash','preferences','applied',?2)",params![db.id.as_str(),huge]).unwrap_err();
        assert_eq!(AppError::from(error).code, ErrorCode::StorageFull);
        drop(tx);
        assert_eq!(db.preferences().unwrap().revision, 0);
        integrity(&db.connection).unwrap();
        let other = open_connection(&root.join("memoquill.sqlite3"), Access::ReadWrite).unwrap();
        other.execute_batch("BEGIN IMMEDIATE").unwrap();
        db.connection.busy_timeout(Duration::ZERO).unwrap();
        let error = db
            .connection
            .execute("UPDATE preferences SET revision=99", [])
            .unwrap_err();
        assert_eq!(AppError::from(error).code, ErrorCode::Busy);
        other.execute_batch("ROLLBACK").unwrap();
        assert_eq!(db.preferences().unwrap().revision, 0);
    }

    #[test]
    fn engine_features_and_actual_foreign_key_rejection() {
        let temp = tempfile::tempdir().unwrap();
        let db = Database::open(&temp.path().join("workspace")).unwrap();
        let version: String = db
            .connection
            .query_row("SELECT sqlite_version()", [], |r| r.get(0))
            .unwrap();
        let source: String = db
            .connection
            .query_row("SELECT sqlite_source_id()", [], |r| r.get(0))
            .unwrap();
        let options = db
            .connection
            .prepare("PRAGMA compile_options")
            .unwrap()
            .query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap();
        println!(
            "SQLITE_RUNTIME {}",
            serde_json::json!({"version":version,"sourceId":source,"compileOptions":options})
        );
        assert_eq!(
            db.connection
                .query_row("SELECT json_valid('{\"a\":1}')", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            1
        );
        db.connection.execute_batch("CREATE VIRTUAL TABLE temp.fts_probe USING fts5(text); INSERT INTO fts_probe VALUES ('quantum cooking');").unwrap();
        assert_eq!(
            db.connection
                .query_row(
                    "SELECT count(*) FROM fts_probe WHERE fts_probe MATCH 'quantum'",
                    [],
                    |r| r.get::<_, i64>(0)
                )
                .unwrap(),
            1
        );
        let error = db
            .connection
            .execute(
                "INSERT INTO preferences(workspace_id,revision,data) VALUES ('missing',0,'{}')",
                [],
            )
            .unwrap_err();
        assert_eq!(
            error.sqlite_error_code(),
            Some(rusqlite::ErrorCode::ConstraintViolation)
        );
    }
    #[test]
    fn runtime_and_backup_budget_boundaries_are_explicit() {
        assert!(runtime_policy(3_051_002).is_err());
        assert!(runtime_policy(3_051_003).is_ok());
        assert!(runtime_policy(3_054_001).is_ok());
        assert!(!backup_expired(
            Duration::from_secs(30) - Duration::from_nanos(1)
        ));
        assert!(backup_expired(Duration::from_secs(30)));
    }
    #[test]
    fn reopening_current_schema_does_not_create_upgrade_snapshots() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        drop(Database::open(&root).unwrap());
        drop(Database::open(&root).unwrap());
        assert_eq!(
            std::fs::read_dir(root.join("migration-backups"))
                .unwrap()
                .count(),
            0
        );
    }
    #[test]
    fn all_policy_controls_are_observable_and_independent() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        let db = Database::open(&root).unwrap();
        assert_eq!(
            db.connection
                .query_row("PRAGMA trusted_schema", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert!(
            db.connection
                .execute_batch(r#"CREATE TABLE bad_literal(x DEFAULT ("unknown_literal"))"#)
                .is_err()
        );
        let memory = Connection::open_in_memory().unwrap();
        base_policy(&memory).unwrap();
        assert_eq!(
            runtime_configuration(&memory).unwrap_err().code,
            ErrorCode::StorageFailure
        );
        db.connection
            .pragma_update(None, "foreign_keys", false)
            .unwrap();
        assert_eq!(
            runtime_configuration(&db.connection).unwrap_err().code,
            ErrorCode::StorageFailure
        );
        db.connection
            .execute(
                "INSERT INTO preferences(workspace_id,revision,data) VALUES ('orphan',0,'{}')",
                [],
            )
            .unwrap();
        assert_eq!(
            integrity(&db.connection).unwrap_err().code,
            ErrorCode::RecoveryRequired
        );
        db.connection
            .pragma_update(None, "user_version", 0)
            .unwrap();
        assert_eq!(
            preflight(&root.join("memoquill.sqlite3")).unwrap_err().code,
            ErrorCode::UnsupportedSchema
        );
    }
    #[test]
    fn an_existing_workspace_uses_wal_before_migration_changes() {
        use super::super::faults::{self, Phase};
        let (_temp, root) = legacy_root();
        let probe = root.join("memoquill.sqlite3");
        let observed = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter = observed.clone();
        let db=faults::using(move|phase|{
            if phase==Phase::MigrationBeforeCommit {
                let reader=open_connection(&probe,Access::ReadOnly)?;
                let mode:String=reader.query_row("PRAGMA journal_mode",[],|r|r.get(0))?;
                assert_eq!(mode,"wal","Existing data must use the selected crash-recovery journal before migrations write");
                counter.fetch_add(1,std::sync::atomic::Ordering::Relaxed);
            }
            Ok(())
        },||Database::open(&root)).unwrap();
        assert_eq!(observed.load(std::sync::atomic::Ordering::Relaxed), 1);
        assert_eq!(db.preferences().unwrap().revision, 7);
    }
}
