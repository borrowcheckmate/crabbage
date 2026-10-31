use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use sqlx::sqlite::SqlitePool;

use crate::load_demo;

#[tokio::test]
async fn loader_creates_a_consistent_idempotent_database_in_both_languages() {
    for language in ["en", "nl"] {
        let path = data_path(language);
        load_demo(&path, language).await.expect("demo data loads");
        let pool = crabbage_store::open(&path).await.expect("database opens");
        assert_fixture_counts(&pool, &path).await;
        assert_language(&pool, language).await;
        load_demo(&path, language)
            .await
            .expect("repeat load succeeds");
        assert_fixture_counts(&pool, &path).await;
        drop(pool);
        std::fs::remove_dir_all(path).expect("temporary fixture is removed");
    }
}

fn data_path(language: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock is after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "crabbage-demo-test-{}-{language}-{nonce}",
        std::process::id()
    ))
}

async fn assert_fixture_counts(pool: &SqlitePool, data_dir: &Path) {
    assert_eq!(count(pool, "recipes").await, 12);
    assert_eq!(count(pool, "recipe_lines").await, 36);
    assert_eq!(count(pool, "steps").await, 24);
    assert_eq!(count(pool, "recipe_categories").await, 12);
    assert_eq!(count(pool, "ratings").await, 36);
    assert_eq!(count(pool, "favourites").await, 18);
    assert_eq!(count(pool, "plan_entries").await, 14);
    assert_eq!(count(pool, "shopping_items").await, 44);
    assert_eq!(count(pool, "users").await, 3);
    assert_eq!(count(pool, "households").await, 2);
    assert_eq!(count(pool, "user_households").await, 3);
    let linked: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM shopping_items si JOIN plan_entries pe ON pe.id = si.plan_entry_id WHERE si.household_id = pe.household_id",
    )
    .fetch_one(pool)
    .await
    .expect("plan shopping links count");
    assert_eq!(linked, 42);
    assert_current_and_next_week(pool).await;
    let images: Vec<String> =
        sqlx::query_scalar("SELECT image FROM recipes WHERE image IS NOT NULL")
            .fetch_all(pool)
            .await
            .expect("recipe image names read");
    assert_eq!(images.len(), 12);
    for name in images {
        let bytes = std::fs::read(data_dir.join("images").join(name)).expect("recipe image exists");
        assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
    }
    let states: (i64, i64) =
        sqlx::query_as("SELECT SUM(checked = 1), SUM(checked = 0) FROM shopping_items")
            .fetch_one(pool)
            .await
            .expect("shopping check states count");
    assert!(states.0 > 0 && states.1 > 0);
}

async fn assert_current_and_next_week(pool: &SqlitePool) {
    let bounds: (String, String) = sqlx::query_as("SELECT MIN(date), MAX(date) FROM plan_entries")
        .fetch_one(pool)
        .await
        .expect("plan dates read");
    let today = i64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock is after epoch")
            .as_secs()
            / 86_400,
    )
    .expect("today fits the date domain");
    let monday = today - (today + 3).rem_euclid(7);
    assert_eq!(
        bounds.0,
        crabbage_domain::dates::format_iso_date(monday).expect("Monday formats")
    );
    assert_eq!(
        bounds.1,
        crabbage_domain::dates::format_iso_date(monday + 13).expect("Sunday formats")
    );
}

async fn count(pool: &SqlitePool, table: &str) -> i64 {
    let query = match table {
        "recipes" => "SELECT COUNT(*) FROM recipes",
        "recipe_lines" => "SELECT COUNT(*) FROM recipe_lines",
        "steps" => "SELECT COUNT(*) FROM steps",
        "recipe_categories" => "SELECT COUNT(*) FROM recipe_categories",
        "ratings" => "SELECT COUNT(*) FROM ratings",
        "favourites" => "SELECT COUNT(*) FROM favourites",
        "plan_entries" => "SELECT COUNT(*) FROM plan_entries",
        "shopping_items" => "SELECT COUNT(*) FROM shopping_items",
        "users" => "SELECT COUNT(*) FROM users",
        "households" => "SELECT COUNT(*) FROM households",
        "user_households" => "SELECT COUNT(*) FROM user_households",
        _ => "SELECT 0",
    };
    sqlx::query_scalar(query)
        .fetch_one(pool)
        .await
        .expect("fixture table count")
}

async fn assert_language(pool: &SqlitePool, language: &str) {
    let first: String = sqlx::query_scalar("SELECT title FROM recipes ORDER BY id LIMIT 1")
        .fetch_one(pool)
        .await
        .expect("recipe title reads");
    let expected = if language == "nl" {
        "Citroen-kruidenpasta"
    } else {
        "Lemon herb pasta"
    };
    assert_eq!(first, expected);
}
