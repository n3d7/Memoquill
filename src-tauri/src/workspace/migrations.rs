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
    super::faults::check(super::faults::Phase::MigrationBeforeCommit)?;
    tx.commit()?;
    Ok(())
}
