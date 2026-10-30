use std::path::Path;

use sqlx::Transaction;
use sqlx::migrate::Migrator;
use sqlx::sqlite::{Sqlite, SqliteConnectOptions, SqliteJournalMode, SqlitePool};

use crate::StoreError;

const DATABASE_FILE: &str = "crabbage.db";
static MIGRATOR: Migrator = sqlx::migrate!();

/// Opens `crabbage.db` in `data_dir`, creating the directory and the database if missing,
/// and applies every pending migration.
///
/// The database uses WAL journaling and enforces foreign keys on every connection.
///
/// # Errors
///
/// Returns an error if the directory cannot be created, the database cannot be opened,
/// or a migration fails.
pub async fn open(data_dir: &Path) -> Result<SqlitePool, StoreError> {
    std::fs::create_dir_all(data_dir)?;
    let options = SqliteConnectOptions::new()
        .filename(data_dir.join(DATABASE_FILE))
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true);
    let pool = SqlitePool::connect_with(options).await?;
    MIGRATOR.run(&pool).await?;
    Ok(pool)
}

/// Begins a transaction that takes the write lock at once. A deferred transaction that reads and
/// then writes fails with a locked database when another connection wrote in between; this one
/// waits for the busy timeout instead.
pub(crate) async fn begin_write(
    pool: &SqlitePool,
) -> Result<Transaction<'static, Sqlite>, sqlx::Error> {
    pool.begin_with("BEGIN IMMEDIATE").await
}
