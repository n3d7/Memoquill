use super::{AppError, ErrorCode, error::Result};
use rusqlite::{Connection, TransactionBehavior, params};
use sha2::{Digest, Sha256};

pub(super) struct Migration {
    pub version: i64,
    pub sql: &'static str,
}

pub(super) const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        sql: include_str!("migrations/001_workspace.sql"),
    },
    Migration {
        version: 2,
        sql: include_str!("migrations/002_operations_assets.sql"),
    },
    Migration {
        version: 3,
        sql: include_str!("migrations/003_content.sql"),
    },
    Migration {
        version: 4,
        sql: include_str!("migrations/004_sources.sql"),
    },
    Migration {
        version: 5,
        sql: include_str!("migrations/005_lessons.sql"),
    },
];

pub(super) fn checksum(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(super) fn validate(connection: &Connection, version: i64) -> Result<()> {
    let mut query =
        connection.prepare("SELECT version, checksum FROM schema_migrations ORDER BY version")?;
    let actual = query
        .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let expected: Vec<_> = MIGRATIONS
        .iter()
        .take(version as usize)
        .map(|m| (m.version, checksum(m.sql.as_bytes())))
        .collect();
    if actual != expected {
        return Err(AppError::new(ErrorCode::RecoveryRequired));
    }
    Ok(())
}

pub(super) fn apply(
    connection: &mut Connection,
    from: i64,
    migrations: &[Migration],
) -> Result<()> {
    // SQLite table rebuilds require foreign keys disabled outside the transaction.
    // Validate every reference before commit, and restore enforcement on every outcome.
    let foreign_keys: bool = connection.pragma_query_value(None, "foreign_keys", |r| r.get(0))?;
    connection.pragma_update(None, "foreign_keys", false)?;
    let result = apply_transaction(connection, from, migrations);
    connection.pragma_update(None, "foreign_keys", foreign_keys)?;
    result
}
fn apply_transaction(
    connection: &mut Connection,
    from: i64,
    migrations: &[Migration],
) -> Result<()> {
    let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let mut previous = from;
    for migration in migrations.iter().filter(|m| m.version > from) {
        if migration.version != previous + 1 {
            return Err(AppError::new(ErrorCode::RecoveryRequired));
        }
        tx.execute_batch(migration.sql)?;
        tx.execute(
            "INSERT INTO schema_migrations(version,checksum) VALUES (?1,?2)",
            params![migration.version, checksum(migration.sql.as_bytes())],
        )?;
        tx.pragma_update(None, "user_version", migration.version)?;
        previous = migration.version;
    }
    let invalid_references: i64 =
        tx.query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| {
            r.get(0)
        })?;
    if invalid_references != 0 {
        return Err(AppError::new(ErrorCode::RecoveryRequired));
    }
    super::faults::check(super::faults::Phase::MigrationBeforeCommit)?;
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod lesson_upgrade_tests {
    use super::*;
    #[test]
    fn schema_four_assets_sources_and_receipts_survive_rebuild_with_enforcement_restored() {
        let mut c = Connection::open_in_memory().unwrap();
        c.pragma_update(None, "foreign_keys", true).unwrap();
        apply(&mut c, 0, &MIGRATIONS[..4]).unwrap();
        c.execute("INSERT INTO workspace_meta VALUES(1,'workspace')", [])
            .unwrap();
        c.execute("INSERT INTO operations VALUES('op','workspace','local-ui','hash','asset','applied','{}',NULL)",[]).unwrap();
        c.execute("INSERT INTO assets VALUES(?1,4,'op')", ["a".repeat(64)])
            .unwrap();
        c.execute("INSERT INTO resources VALUES('resource','Original',0)", [])
            .unwrap();
        c.execute("INSERT INTO original_sources VALUES('source','resource',?1,'source.pdf','application/pdf','Author','CC0',NULL,1,4)",["a".repeat(64)]).unwrap();
        apply(&mut c, 4, MIGRATIONS).unwrap();
        assert!(
            c.pragma_query_value(None, "foreign_keys", |r| r.get::<_, bool>(0))
                .unwrap()
        );
        assert_eq!(c.query_row("SELECT count(*) FROM original_sources s JOIN assets a ON a.digest=s.digest JOIN operations o ON o.id=a.registered_by",[],|r|r.get::<_,i64>(0)).unwrap(),1);
        assert!(
            c.execute(
                "INSERT INTO assets VALUES(?1,1,'missing')",
                ["b".repeat(64)]
            )
            .is_err()
        );
        validate(&c, 5).unwrap();
    }
    #[test]
    fn failed_upgrade_rolls_back_version_and_restores_foreign_key_enforcement() {
        let mut c = Connection::open_in_memory().unwrap();
        c.pragma_update(None, "foreign_keys", true).unwrap();
        apply(&mut c, 0, MIGRATIONS).unwrap();
        let bad = [Migration {
            version: 6,
            sql: "CREATE TABLE broken(id TEXT REFERENCES resources(id)); INSERT INTO broken VALUES('missing');",
        }];
        assert_eq!(
            apply(&mut c, 5, &bad).unwrap_err().code,
            ErrorCode::RecoveryRequired
        );
        assert!(
            c.pragma_query_value(None, "foreign_keys", |r| r.get::<_, bool>(0))
                .unwrap()
        );
        assert_eq!(
            c.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
                .unwrap(),
            5
        );
        assert_eq!(
            c.query_row(
                "SELECT count(*) FROM sqlite_master WHERE name='broken'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
            0
        );
        validate(&c, 5).unwrap();
    }
}
