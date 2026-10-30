//! Opening the database creates the file, applies migrations and sets the connection pragmas.

use std::path::{Path, PathBuf};

use crabbage_store::{StoreError, open};
use sqlx::Row;
use sqlx::sqlite::SqlitePool;

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("crabbage-store-{}-{name}", std::process::id()));
        drop(std::fs::remove_dir_all(&path));
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        drop(std::fs::remove_dir_all(&self.0));
    }
}

async fn migration_count(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(pool)
        .await
}

#[tokio::test]
async fn fresh_dir_gets_a_migrated_database() {
    let dir = TempDir::new("fresh");
    let pool = open(dir.path()).await.unwrap();
    assert!(dir.path().join("crabbage.db").is_file());
    assert_eq!(migration_count(&pool).await.unwrap(), 4);
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name IN ('households', 'users', 'user_households') ORDER BY name",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(tables, ["households", "user_households", "users"]);
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name IN ('login_attempts', 'sessions') ORDER BY name",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(tables, ["login_attempts", "sessions"]);
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name IN ('ingredient_aliases', 'ingredients', 'recipe_lines', 'recipes', 'store_sections') ORDER BY name",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        tables,
        [
            "ingredient_aliases",
            "ingredients",
            "recipe_lines",
            "recipes",
            "store_sections"
        ]
    );
}

#[tokio::test]
async fn opening_twice_keeps_the_migration_count() {
    let dir = TempDir::new("twice");
    drop(open(dir.path()).await.unwrap());
    let pool = open(dir.path()).await.unwrap();
    assert_eq!(migration_count(&pool).await.unwrap(), 4);
}

#[tokio::test]
async fn wal_and_foreign_keys_are_on() {
    let dir = TempDir::new("pragmas");
    let pool = open(dir.path()).await.unwrap();
    let journal: String = sqlx::query_scalar("PRAGMA journal_mode")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(journal, "wal");
    let foreign_keys: i64 = sqlx::query("PRAGMA foreign_keys")
        .fetch_one(&pool)
        .await
        .unwrap()
        .get(0);
    assert_eq!(foreign_keys, 1);
}

#[tokio::test]
async fn nested_missing_dir_is_created() {
    let dir = TempDir::new("nested");
    let nested = dir.path().join("a").join("b");
    drop(open(&nested).await.unwrap());
    assert!(nested.join("crabbage.db").is_file());
}

#[tokio::test]
async fn data_dir_below_a_file_is_an_io_error() {
    let dir = TempDir::new("blocked");
    std::fs::create_dir(dir.path()).unwrap();
    let blocker = dir.path().join("blocker");
    std::fs::write(&blocker, b"").unwrap();
    let error = open(&blocker.join("data")).await.err().unwrap();
    assert!(matches!(error, StoreError::Io(_)), "{error}");
}
