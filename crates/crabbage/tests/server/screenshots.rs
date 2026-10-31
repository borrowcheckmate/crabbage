//! Writes the rendered pages to `CRABBAGE_SCREENSHOT_DIR` so they can be photographed for review.
//! The stylesheet and the sample photo are copied next to the pages and linked relatively, so a
//! `file://` view loads them.
//! Ignored by default: run with `cargo test --features crabbage/ssr --test server -- --ignored`.

use std::path::{Path, PathBuf};

use axum::http::StatusCode;
use crabbage_store::catalog::{categories, ingredients, sections};
use crabbage_store::households::households_of;
use crabbage_store::planner::{PlanEntry, PlannerSettings, save_settings, set_entry};
use crabbage_store::recipes::{LineInput, RecipeInput, StepInput, create, set_image};
use crabbage_store::shopping::add_text;

use crate::common::{Harness, body_text, session_token, sign_in_as};
use crate::recipe_import::serve_with;
use crate::recipe_photo_import::{location, png, upload, with_ai};

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "the writer lists every page it photographs, one line each"
)]
#[ignore = "writes review screenshots' HTML; needs CRABBAGE_SCREENSHOT_DIR"]
async fn write_pages() {
    let dir = PathBuf::from(std::env::var("CRABBAGE_SCREENSHOT_DIR").expect("dir is set"));
    std::fs::create_dir_all(&dir).expect("dir is created");
    let css = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/style/main.css"))
        .expect("stylesheet is read");
    std::fs::write(dir.join("crabbage.css"), css).expect("stylesheet is written");
    let harness = Harness::start_with(
        "screenshots",
        None,
        &[("CRABBAGE_IMPORT_ALLOW_PRIVATE", "true")],
    )
    .await;
    let source = serve_with(imported_page).await;
    let produce = sections::create(&harness.pool, "Produce")
        .await
        .expect("section");
    let dairy = sections::create(&harness.pool, "Dairy")
        .await
        .expect("section");
    let tomato = ingredients::create(&harness.pool, "Tomato")
        .await
        .expect("ingredient");
    let onion = ingredients::create(&harness.pool, "Onion")
        .await
        .expect("ingredient");
    ingredients::set_section(&harness.pool, tomato, Some(produce))
        .await
        .expect("section set");
    ingredients::set_section(&harness.pool, onion, Some(dairy))
        .await
        .expect("section set");
    ingredients::add_alias(&harness.pool, tomato, "Pomodoro")
        .await
        .expect("alias");
    sqlx::query("INSERT INTO ingredients (name, pending_review) VALUES ('Okra', 1)")
        .execute(&harness.pool)
        .await
        .expect("pending ingredient");

    harness.issuer.set_groups(&[]);
    harness.issuer.set_name(Some("Sam Baker"));
    let member = session_token(&harness.sign_in().await).expect("sign-in sets a session");
    let shopping_token =
        sign_in_as(&harness, "shopper", &["crabbage-household-screenshot-home"]).await;
    let shopper: i64 = sqlx::query_scalar("SELECT id FROM users WHERE subject = 'shopper'")
        .fetch_one(&harness.pool)
        .await
        .expect("shopper exists");
    let household = households_of(&harness.pool, shopper)
        .await
        .expect("households")
        .first()
        .expect("household")
        .id;
    let owner = harness.user_id().await;
    let breakfast = categories::create(&harness.pool, "Breakfast")
        .await
        .expect("category");
    categories::create(&harness.pool, "Dinner")
        .await
        .expect("category");
    let recipe = create(&harness.pool, owner, &sample_recipe(breakfast), 0)
        .await
        .expect("recipe");
    save_settings(
        &harness.pool,
        household,
        &PlannerSettings {
            lookback_weeks: 2,
            servings: 4,
            weekdays: [
                Some(breakfast),
                Some(breakfast),
                None,
                Some(breakfast),
                Some(breakfast),
                None,
                Some(breakfast),
            ],
        },
    )
    .await
    .expect("planner settings");
    for date in [
        "2024-02-26",
        "2024-02-27",
        "2024-02-28",
        "2024-02-29",
        "2024-03-01",
        "2024-03-02",
        "2024-03-03",
    ] {
        set_entry(
            &harness.pool,
            household,
            &PlanEntry {
                date: date.to_owned(),
                recipe_id: Some(recipe),
                servings: 4,
            },
        )
        .await
        .expect("plan entry");
    }
    let today = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock is after the epoch")
        .as_secs()
        / 86_400;
    let today = crabbage_domain::dates::format_iso_date(i64::try_from(today).expect("date fits"))
        .expect("date is valid");
    set_entry(
        &harness.pool,
        household,
        &PlanEntry {
            date: today.clone(),
            recipe_id: Some(recipe),
            servings: 4,
        },
    )
    .await
    .expect("today entry");
    for ingredient in [tomato, onion] {
        sqlx::query("INSERT INTO shopping_items (household_id, ingredient_id) VALUES (?, ?)")
            .bind(household)
            .bind(ingredient)
            .execute(&harness.pool)
            .await
            .expect("section item");
    }
    add_text(&harness.pool, household, "Paper towels")
        .await
        .expect("manual item");
    sqlx::query("UPDATE shopping_items SET checked = 1 WHERE ingredient_id = ?")
        .bind(tomato)
        .execute(&harness.pool)
        .await
        .expect("checked item");
    let shopping_body = body_text(harness.get("/shopping", Some(&shopping_token)).await)
        .await
        .replace("href=\"/pkg/crabbage.css\"", "href=\"crabbage.css\"");
    std::fs::write(dir.join("member-shopping.html"), shopping_body).expect("page is written");
    let today_body = body_text(harness.get("/", Some(&shopping_token)).await)
        .await
        .replace("href=\"/pkg/crabbage.css\"", "href=\"crabbage.css\"");
    std::fs::write(dir.join("member-today.html"), today_body).expect("page is written");
    write_today_review_pages(&dir, &harness, household, &today, recipe, &shopping_token).await;
    let plan_body = body_text(
        harness
            .get("/plan?week=2024-02-26", Some(&shopping_token))
            .await,
    )
    .await
    .replace("href=\"/pkg/crabbage.css\"", "href=\"crabbage.css\"");
    std::fs::write(dir.join("member-plan.html"), plan_body).expect("page is written");
    std::fs::create_dir_all(harness.images_dir()).expect("images directory is created");
    std::fs::write(harness.images_dir().join(PHOTO_NAME), PHOTO).expect("photo is written");
    set_image(&harness.pool, recipe, Some(PHOTO_NAME))
        .await
        .expect("photo is set");
    std::fs::create_dir_all(dir.join("images")).expect("images directory is created");
    std::fs::copy(
        harness.images_dir().join(PHOTO_NAME),
        dir.join("images").join(PHOTO_NAME),
    )
    .expect("photo is copied");
    harness.issuer.set_name(Some("Alex Baker"));
    let neighbour = sign_in_as(&harness, "neighbour", &[]).await;
    harness.issuer.set_name(Some("Sam Baker"));
    harness.issuer.set_subject("subject-1");
    harness
        .post_form(
            &format!("/recipes/{recipe}/rating"),
            Some(&neighbour),
            "stars=5",
        )
        .await;
    harness
        .post_form(
            &format!("/recipes/{recipe}/rating"),
            Some(&member),
            "stars=4",
        )
        .await;
    harness
        .post_form(
            &format!("/recipes/{recipe}/favourite"),
            Some(&member),
            "on=true",
        )
        .await;
    let import_path = format!("/recipes/import?url={source}/recipe");
    let recipe_path = format!("/recipes/{recipe}");
    let breakfast_path = format!("/recipes?category={breakfast}");
    let recipe_edit_path = format!("/recipes/{recipe}/edit");
    let ingredient_path = format!("/admin/ingredients/{tomato}");
    let ingredient_delete_path = format!("{ingredient_path}?confirm=delete");
    let recipe_delete_path = format!("{recipe_path}?confirm=delete");
    let recipe_servings_path = format!("{recipe_path}?servings=8");
    let cook_path = format!("{recipe_path}/cook?step=2&done=1");
    let cook_timer_path = format!("{recipe_path}/cook?step=3");
    let mealie_admin = sign_in_as(&harness, "screenshot-mealie-admin", &["admin"]).await;
    let mealie = crate::mealie::mealie_server(false).await;
    crate::mealie::start(&harness, &mealie_admin, &mealie, "fixture-token").await;
    crate::mealie::page_until(&harness, &mealie_admin, "Failed: 1").await;

    let pages = [
        ("admin", &["admin"][..], "home", "/"),
        ("admin", &["admin"][..], "ingredients", "/admin/ingredients"),
        ("admin", &["admin"][..], "sections", "/admin/sections"),
        ("admin", &["admin"][..], "review", "/admin/review"),
        ("admin", &["admin"][..], "categories", "/admin/categories"),
        ("admin", &["admin"][..], "mealie", "/admin/mealie"),
        (
            "admin",
            &["admin"][..],
            "ingredient",
            ingredient_path.as_str(),
        ),
        (
            "admin",
            &["admin"][..],
            "ingredient-delete",
            ingredient_delete_path.as_str(),
        ),
        (
            "admin",
            &["admin"][..],
            "ingredients-search",
            "/admin/ingredients?q=pom",
        ),
        ("member", &[][..], "home", "/"),
        ("member", &[][..], "ingredients", "/admin/ingredients"),
        ("member", &[][..], "recipes", "/recipes"),
        (
            "member",
            &[][..],
            "recipes-favourites",
            "/recipes?favourites=1",
        ),
        (
            "member",
            &[][..],
            "recipes-breakfast",
            breakfast_path.as_str(),
        ),
        ("member", &[][..], "recipe", recipe_path.as_str()),
        ("member", &[][..], "recipe-edit", recipe_edit_path.as_str()),
        (
            "member",
            &[][..],
            "recipe-delete",
            recipe_delete_path.as_str(),
        ),
        ("member", &[][..], "recipe-new", "/recipes/new"),
        ("member", &[][..], "recipe-import", "/recipes/import"),
        (
            "member",
            &[][..],
            "recipe-import-preview",
            import_path.as_str(),
        ),
        ("member", &[][..], "recipes-search", "/recipes?q=flour"),
        ("member", &[][..], "not-found", "/not-a-page"),
        (
            "member",
            &[][..],
            "recipe-servings",
            recipe_servings_path.as_str(),
        ),
        ("member", &[][..], "cook", cook_path.as_str()),
        ("member", &[][..], "cook-timer", cook_timer_path.as_str()),
    ];
    for (who, groups, name, path) in pages {
        harness.issuer.set_groups(groups);
        let token = session_token(&harness.sign_in().await).expect("sign-in sets a session");
        let body = body_text(harness.get(path, Some(&token)).await)
            .await
            .replace("href=\"/pkg/crabbage.css\"", "href=\"crabbage.css\"")
            .replace("src=\"/images/", "src=\"images/")
            .replace(
                &format!("{source}/photo.png"),
                &format!("images/{PHOTO_NAME}"),
            );
        std::fs::write(dir.join(format!("{who}-{name}.html")), body).expect("page is written");
    }
    let (with_ai, _requests) = with_ai("screenshots-ai", StatusCode::OK).await;
    let token = sign_in_as(&with_ai, "member", &[]).await;
    let body = body_text(with_ai.get("/recipes/import", Some(&token)).await)
        .await
        .replace("href=\"/pkg/crabbage.css\"", "href=\"crabbage.css\"");
    std::fs::write(dir.join("member-recipe-import-photos.html"), body).expect("page is written");
    let photo = png();
    let review = location(&upload(&with_ai, Some(&token), &[("photo", &photo)]).await);
    let body = body_text(with_ai.get(&review, Some(&token)).await)
        .await
        .replace("href=\"/pkg/crabbage.css\"", "href=\"crabbage.css\"");
    std::fs::write(dir.join("member-recipe-import-photo-review.html"), body)
        .expect("page is written");
}

#[expect(
    clippy::expect_used,
    reason = "the screenshot writer stops when a page or fixture cannot be written"
)]
async fn write_today_review_pages(
    dir: &Path,
    harness: &Harness,
    household: i64,
    today: &str,
    recipe: i64,
    shopping_token: &str,
) {
    sqlx::query("DELETE FROM plan_entries WHERE household_id = ? AND date = ?")
        .bind(household)
        .bind(today)
        .execute(&harness.pool)
        .await
        .expect("today entry is cleared");
    let empty_today = body_text(harness.get("/", Some(shopping_token)).await)
        .await
        .replace("href=\"/pkg/crabbage.css\"", "href=\"crabbage.css\"");
    std::fs::write(dir.join("member-today-empty.html"), empty_today)
        .expect("empty Today page is written");
    set_entry(
        &harness.pool,
        household,
        &PlanEntry {
            date: today.to_owned(),
            recipe_id: Some(recipe),
            servings: 4,
        },
    )
    .await
    .expect("today entry is restored");
    let dutch = Harness::start_with_language("screenshots-nl", Some("nl")).await;
    let dutch_token = sign_in_as(
        &dutch,
        "dutch-member",
        &["crabbage-household-screenshot-nl"],
    )
    .await;
    for (name, path) in [
        ("member-today-nl.html", "/"),
        ("member-shopping-nl.html", "/shopping"),
    ] {
        let body = body_text(dutch.get(path, Some(&dutch_token)).await)
            .await
            .replace("href=\"/pkg/crabbage.css\"", "href=\"crabbage.css\"");
        std::fs::write(dir.join(name), body).expect("Dutch page is written");
    }
}

const PHOTO_NAME: &str = "5a0c1e2f3b4d5e6f7a8b9c0d1e2f3a4b.png";

/// A 4 by 4 pixel PNG of one colour.
const PHOTO: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x04, 0x08, 0x02, 0x00, 0x00, 0x00, 0x26, 0x93, 0x09,
    0x29, 0x00, 0x00, 0x00, 0x10, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x38, 0x11, 0x6A, 0x0B,
    0x47, 0x0C, 0xC4, 0x71, 0x00, 0x3B, 0x42, 0x15, 0xA1, 0x4E, 0x94, 0x8D, 0x55, 0x00, 0x00, 0x00,
    0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

/// A recipe with four lines and three steps, owned by the member who signs in first and filed under
/// `category`.
fn sample_recipe(category: i64) -> RecipeInput {
    let line =
        |amount: Option<f64>, unit: Option<&str>, ingredient: &str, note: Option<&str>| LineInput {
            amount,
            unit: unit.map(str::to_owned),
            ingredient: Some(ingredient.to_owned()),
            note: note.map(str::to_owned),
        };
    RecipeInput {
        title: "Pancakes".to_owned(),
        servings: 4,
        lines: vec![
            line(Some(250.0), Some("g"), "Flour", Some("sifted")),
            line(Some(2.0), Some("piece"), "Egg", None),
            line(None, Some("to taste"), "Salt", None),
            line(Some(0.5), Some("tsp"), "Vanilla extract", None),
        ],
        steps: vec![
            StepInput {
                text: "Whisk the eggs with the milk.".to_owned(),
                duration_seconds: Some(120),
                line_positions: vec![1],
            },
            StepInput {
                text: "Stir in the flour and a pinch of salt.".to_owned(),
                duration_seconds: None,
                line_positions: vec![0, 2],
            },
            StepInput {
                text: "Fry for 4 minutes until golden on both sides.".to_owned(),
                duration_seconds: Some(240),
                line_positions: vec![],
            },
        ],
        category_ids: vec![category],
    }
}

/// A recipe page to import: one ingredient line is not in the catalogue. The photo is served from
/// `base`.
fn imported_page(base: &str) -> String {
    let recipe = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "Recipe",
        "name": "Lemon ricotta pancakes",
        "image": format!("{base}/photo.png"),
        "recipeYield": "4 servings",
        "recipeIngredient": ["250 g Flour", "2 Egg", "1 tsp Vanilla extract", "200 g Ricotta, drained"],
        "recipeInstructions": [
            {"@type": "HowToStep", "text": "Mix the flour, eggs and ricotta into a thick batter."},
            {"@type": "HowToStep", "text": "Fry spoonfuls of batter until golden on both sides."}
        ]
    });
    format!(
        "<html lang=\"en\"><head><script type=\"application/ld+json\">{recipe}</script></head></html>"
    )
}
