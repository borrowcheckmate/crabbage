//! Writes the rendered pages to `CRABBAGE_SCREENSHOT_DIR` so they can be photographed for review.
//! The stylesheet and the sample photo are copied next to the pages and linked relatively, so a
//! `file://` view loads them.
//! Ignored by default: run with `cargo test --features crabbage/ssr --test server -- --ignored`.

use std::path::PathBuf;

use crabbage_store::catalog::{categories, ingredients, sections};
use crabbage_store::recipes::{LineInput, RecipeInput, StepInput, create, set_image};

use crate::common::{Harness, body_text, session_token, sign_in_as};

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
    let harness = Harness::start("screenshots").await;
    let produce = sections::create(&harness.pool, "Produce")
        .await
        .expect("section");
    sections::create(&harness.pool, "Dairy")
        .await
        .expect("section");
    let tomato = ingredients::create(&harness.pool, "Tomato")
        .await
        .expect("ingredient");
    ingredients::create(&harness.pool, "Onion")
        .await
        .expect("ingredient");
    ingredients::set_section(&harness.pool, tomato, Some(produce))
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
    let recipe_path = format!("/recipes/{recipe}");
    let breakfast_path = format!("/recipes?category={breakfast}");
    let recipe_edit_path = format!("/recipes/{recipe}/edit");
    let ingredient_path = format!("/admin/ingredients/{tomato}");
    let ingredient_delete_path = format!("{ingredient_path}?confirm=delete");
    let recipe_delete_path = format!("{recipe_path}?confirm=delete");

    let pages = [
        ("admin", &["crabbage-admin"][..], "home", "/"),
        (
            "admin",
            &["crabbage-admin"][..],
            "ingredients",
            "/admin/ingredients",
        ),
        (
            "admin",
            &["crabbage-admin"][..],
            "sections",
            "/admin/sections",
        ),
        ("admin", &["crabbage-admin"][..], "review", "/admin/review"),
        (
            "admin",
            &["crabbage-admin"][..],
            "categories",
            "/admin/categories",
        ),
        (
            "admin",
            &["crabbage-admin"][..],
            "ingredient",
            ingredient_path.as_str(),
        ),
        (
            "admin",
            &["crabbage-admin"][..],
            "ingredient-delete",
            ingredient_delete_path.as_str(),
        ),
        (
            "admin",
            &["crabbage-admin"][..],
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
    ];
    for (who, groups, name, path) in pages {
        harness.issuer.set_groups(groups);
        let token = session_token(&harness.sign_in().await).expect("sign-in sets a session");
        let body = body_text(harness.get(path, Some(&token)).await)
            .await
            .replace("href=\"/pkg/crabbage.css\"", "href=\"crabbage.css\"")
            .replace("src=\"/images/", "src=\"images/");
        std::fs::write(dir.join(format!("{who}-{name}.html")), body).expect("page is written");
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
                text: "Fry in a hot pan until golden on both sides.".to_owned(),
                duration_seconds: Some(240),
                line_positions: vec![],
            },
        ],
        category_ids: vec![category],
    }
}
