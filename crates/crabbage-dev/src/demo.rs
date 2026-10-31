//! Idempotent demo fixture generation through the application store.

use std::error::Error;
use std::io;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use image::{DynamicImage, ImageFormat, RgbImage};
use sqlx::sqlite::SqlitePool;

use crate::DemoRecipe;

type DemoResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

/// Seeds a complete demo database and its generated recipe photos. Repeated calls leave it intact.
///
/// # Errors
///
/// Returns an error if the language is unsupported, the data directory cannot be accessed, or a
/// store operation or image write fails.
pub async fn load_demo(data_dir: &Path, language: &str) -> DemoResult<()> {
    if !matches!(language, "en" | "nl") {
        return Err(
            io::Error::new(io::ErrorKind::InvalidInput, "language must be en or nl").into(),
        );
    }
    let pool = crabbage_store::open(data_dir).await?;
    crabbage_store::seed_defaults(&pool, language).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS demo_seed (id INTEGER PRIMARY KEY CHECK (id = 1))")
        .execute(&pool)
        .await?;
    let seeded: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM demo_seed WHERE id = 1)")
        .fetch_one(&pool)
        .await?;
    if seeded {
        return Ok(());
    }
    seed_users(&pool).await?;
    let recipes = seed_recipes(&pool, data_dir, language).await?;
    seed_ratings(&pool, &recipes).await?;
    seed_plan(&pool, &recipes).await?;
    sqlx::query("INSERT INTO demo_seed (id) VALUES (1)")
        .execute(&pool)
        .await?;
    Ok(())
}

/// Turns AI on for the demo, pointing at an `OpenAI` compatible endpoint without a key.
///
/// # Errors
///
/// Returns an error when the database cannot be opened or written.
pub async fn enable_demo_ai(data_dir: &Path, base_url: &str) -> DemoResult<()> {
    let pool = crabbage_store::open(data_dir).await?;
    let settings = crabbage_store::settings::AiSettings {
        ai_enabled: true,
        provider: "custom".to_owned(),
        base_url: base_url.to_owned(),
        model: "demo".to_owned(),
        api_key: None,
    };
    crabbage_store::settings::save(&pool, &settings).await?;
    Ok(())
}

async fn seed_users(pool: &SqlitePool) -> DemoResult<()> {
    let admin = crabbage_store::users::upsert_user(pool, "demo-admin", "Demo Admin").await?;
    crabbage_store::households::sync_membership(
        pool,
        admin,
        true,
        &[
            "crabbage-household-maple".to_owned(),
            "crabbage-household-birch".to_owned(),
        ],
    )
    .await?;
    let member = crabbage_store::users::upsert_user(pool, "demo-member", "Demo Member").await?;
    crabbage_store::households::sync_membership(
        pool,
        member,
        false,
        &["crabbage-household-maple".to_owned()],
    )
    .await?;
    let solo = crabbage_store::users::upsert_user(pool, "demo-solo", "Demo Solo").await?;
    crabbage_store::households::sync_membership(pool, solo, false, &[]).await?;
    Ok(())
}

async fn seed_recipes(pool: &SqlitePool, data_dir: &Path, language: &str) -> DemoResult<Vec<i64>> {
    let owner = crabbage_store::users::upsert_user(pool, "demo-admin", "Demo Admin").await?;
    let categories = crabbage_store::catalog::categories::list(pool).await?;
    let mut ids = Vec::new();
    for (index, recipe) in DemoRecipe::all().iter().enumerate() {
        let category = categories.iter().cycle().nth(index).map(|item| item.id);
        let input = recipe.input(language, category);
        let id = crabbage_store::recipes::create(
            pool,
            owner,
            &input,
            1_800_000_000 + i64::try_from(index)?,
        )
        .await?;
        let (name, bytes) = photo(id, index)?;
        tokio::fs::create_dir_all(data_dir.join("images")).await?;
        tokio::fs::write(data_dir.join("images").join(&name), bytes).await?;
        crabbage_store::recipes::set_image(pool, id, Some(&name)).await?;
        ids.push(id);
    }
    Ok(ids)
}

async fn seed_ratings(pool: &SqlitePool, recipes: &[i64]) -> DemoResult<()> {
    let users = ["demo-admin", "demo-member", "demo-solo"];
    for (index, recipe) in recipes.iter().enumerate() {
        for (user_index, subject) in users.iter().enumerate() {
            let user_id: i64 = sqlx::query_scalar("SELECT id FROM users WHERE subject = ?")
                .bind(subject)
                .fetch_one(pool)
                .await?;
            let stars = 3 + i64::try_from((index + user_index) % 3)?;
            crabbage_store::recipes::rate(pool, *recipe, user_id, stars).await?;
            if (index + user_index) % 2 == 0 {
                crabbage_store::recipes::set_favourite(pool, *recipe, user_id, true).await?;
            }
        }
    }
    Ok(())
}

async fn seed_plan(pool: &SqlitePool, recipes: &[i64]) -> DemoResult<()> {
    let household: i64 = sqlx::query_scalar("SELECT id FROM households WHERE group_name = ?")
        .bind("crabbage-household-maple")
        .fetch_one(pool)
        .await?;
    let today = i64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() / 86_400)?;
    let monday = today - (today + 3).rem_euclid(7);
    for offset in 0_i64..14 {
        let date = crabbage_domain::dates::format_iso_date(monday + offset)
            .ok_or_else(|| io::Error::other("date outside supported range"))?;
        crabbage_store::planner::set_entry(
            pool,
            household,
            &crabbage_store::planner::PlanEntry {
                date,
                recipe_id: Some(
                    *recipes
                        .iter()
                        .cycle()
                        .nth(usize::try_from(offset)?)
                        .ok_or_else(|| io::Error::other("recipes are empty"))?,
                ),
                servings: 4,
            },
        )
        .await?;
    }
    let from = crabbage_domain::dates::format_iso_date(monday)
        .ok_or_else(|| io::Error::other("date outside supported range"))?;
    let to = crabbage_domain::dates::format_iso_date(monday + 13)
        .ok_or_else(|| io::Error::other("date outside supported range"))?;
    crabbage_store::shopping::add_plan(pool, household, &from, &to).await?;
    let list = crabbage_store::shopping::list(pool, household).await?;
    let checked: Vec<i64> = list.iter().step_by(3).map(|item| item.id).collect();
    crabbage_store::shopping::set_checked(pool, household, &checked, true).await?;
    crabbage_store::shopping::add_text(pool, household, "Coffee").await?;
    let manual = crabbage_store::shopping::add_text(pool, household, "Dish soap").await?;
    crabbage_store::shopping::set_checked(pool, household, &[manual], true).await?;
    Ok(())
}

/// Muted food colours the demo photos blend between.
const PALETTE: [[u8; 3]; 8] = [
    [196, 120, 74],
    [222, 178, 96],
    [128, 150, 92],
    [176, 74, 60],
    [214, 196, 160],
    [94, 120, 84],
    [232, 146, 104],
    [150, 104, 84],
];

fn photo(id: i64, seed: usize) -> DemoResult<(String, Vec<u8>)> {
    let (width, height) = (192_u32, 128_u32);
    let colour = |step: usize| {
        PALETTE
            .iter()
            .cycle()
            .nth(step)
            .copied()
            .unwrap_or_default()
    };
    let (from, to) = (colour(seed), colour(seed * 3 + 1));
    let mut image = RgbImage::new(width, height);
    let span = width + height;
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let step = x + y;
        for ((channel, a), b) in pixel.0.iter_mut().zip(from).zip(to) {
            *channel = u8::try_from((u32::from(a) * (span - step) + u32::from(b) * step) / span)?;
        }
    }
    let mut bytes = io::Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(image).write_to(&mut bytes, ImageFormat::Png)?;
    Ok((format!("{id:032x}.png"), bytes.into_inner()))
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
