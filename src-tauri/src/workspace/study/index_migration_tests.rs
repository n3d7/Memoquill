use super::*;
use crate::workspace::{database, migrations};
use rusqlite::Connection;

const DOWNGRADE: &str = "DROP TABLE chat_runs;
DROP TABLE chat_threads;
DELETE FROM schema_migrations WHERE version=12;
DROP TABLE agent_authored_documents;
DROP TABLE agent_course_drafts;
DELETE FROM schema_migrations WHERE version=11;
DROP INDEX IF EXISTS study_judgments_attempt;
    DROP INDEX IF EXISTS study_operations_session;
    DROP INDEX IF EXISTS study_sessions_unit;
    DELETE FROM schema_migrations WHERE version=10;
    PRAGMA user_version=9;";

fn populated_v9() -> (tempfile::TempDir, Database, StudyMutation, StudySession) {
    let (temp, mut db, unit, revision) = setup(recall());
    let s = start(&mut db, &unit, &revision, StudyMode::Practice);
    let s = save(
        &mut db,
        &s,
        StudyResponse::Text {
            value: "Answer".into(),
        },
    );
    let submit = request(
        &s,
        StudyAction::Submit {
            self_rating: Some(4),
            recall_rating: None,
            self_outcome: Some(AssessmentOutcome::Correct),
        },
    );
    let saved = db.study_apply(context(), &submit).unwrap();
    db.connection.execute_batch(DOWNGRADE).unwrap();
    (temp, db, submit, saved)
}

fn ledger(connection: &Connection) -> Vec<String> {
    connection.prepare("SELECT json_array('session',id,data,digest) FROM study_sessions
        UNION ALL SELECT json_array('attempt',id,data,digest) FROM study_attempts
        UNION ALL SELECT json_array('judgment',id,data,digest) FROM study_judgments
        UNION ALL SELECT json_array('operation',id,session_id,request_hash,result,digest) FROM study_operations
        ORDER BY 1").unwrap().query_map([], |r| r.get(0)).unwrap()
        .collect::<rusqlite::Result<Vec<_>>>().unwrap()
}

fn index_count(connection: &Connection) -> i64 {
    connection
        .query_row(
            "SELECT count(*) FROM sqlite_schema WHERE type='index'
        AND name IN ('study_judgments_attempt','study_operations_session','study_sessions_unit')",
            [],
            |r| r.get(0),
        )
        .unwrap()
}

#[test]
fn index_migration_keeps_ledger_retries_and_verified_v9_snapshot() {
    let (temp, db, submit, saved) = populated_v9();
    let before = ledger(&db.connection);
    let preferences = serde_json::to_value(db.preferences().unwrap()).unwrap();
    drop(db);
    let root = temp.path().join("workspace");
    let mut db = Database::open(&root).unwrap();
    assert_eq!(index_count(&db.connection), 3);
    assert_eq!(ledger(&db.connection), before);
    assert_eq!(
        serde_json::to_value(db.preferences().unwrap()).unwrap(),
        preferences
    );
    validate_native_data(&db.connection).unwrap();
    assert_eq!(db.study_apply(context(), &submit).unwrap(), saved);
    let snapshots = std::fs::read_dir(root.join("migration-backups"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect::<Vec<_>>();
    assert_eq!(snapshots.len(), 1);
    let snapshot = database::open_connection(&snapshots[0], database::Access::ReadOnly).unwrap();
    database::integrity(&snapshot).unwrap();
    migrations::validate(&snapshot, 9).unwrap();
    assert_eq!(
        snapshot
            .pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
            .unwrap(),
        9
    );
    assert_eq!(index_count(&snapshot), 0);
    assert_eq!(ledger(&snapshot), before);
}

#[test]
fn index_migration_failure_rolls_back_schema_and_retains_history() {
    let (_temp, mut db, _submit, _saved) = populated_v9();
    let before = ledger(&db.connection);
    let result = faults::using(
        |phase| {
            if phase == faults::Phase::MigrationBeforeCommit {
                Err(AppError::new(ErrorCode::StorageFull))
            } else {
                Ok(())
            }
        },
        || migrations::apply(&mut db.connection, 9, migrations::MIGRATIONS),
    );
    assert_eq!(result.unwrap_err().code, ErrorCode::StorageFull);
    assert_eq!(index_count(&db.connection), 0);
    assert_eq!(ledger(&db.connection), before);
    assert_eq!(
        db.connection
            .pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
            .unwrap(),
        9
    );
    assert!(
        db.connection
            .pragma_query_value(None, "foreign_keys", |r| r.get::<_, bool>(0))
            .unwrap()
    );
    migrations::validate(&db.connection, 9).unwrap();
    migrations::apply(&mut db.connection, 9, migrations::MIGRATIONS).unwrap();
    assert_eq!(index_count(&db.connection), 3);
    assert_eq!(ledger(&db.connection), before);
}
