//! Writes the rendered pages to `CRABBAGE_SCREENSHOT_DIR` so they can be photographed for review.
//! The stylesheet is copied next to the pages and linked relatively, so a `file://` view loads it.
//! Ignored by default: run with `cargo test --features crabbage/ssr --test server -- --ignored`.

use std::path::PathBuf;

use crabbage_store::catalog::{ingredients, sections};
use crabbage_store::recipes::{LineInput, RecipeInput, StepInput, create};

use crate::common::{Harness, body_text, session_token};

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
    session_token(&harness.sign_in().await).expect("sign-in sets a session");
    let owner = harness.user_id().await;
    let recipe = create(&harness.pool, owner, &sample_recipe(), 0)
        .await
        .expect("recipe");
    let recipe_path = format!("/recipes/{recipe}");
    let recipe_edit_path = format!("/recipes/{recipe}/edit");
    let ingredient_path = format!("/admin/ingredients/{tomato}");
    let ingredient_delete_path = format!("{ingredient_path}?confirm=delete");

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
        ("member", &[][..], "recipe", recipe_path.as_str()),
        ("member", &[][..], "recipe-edit", recipe_edit_path.as_str()),
        ("member", &[][..], "recipe-new", "/recipes/new"),
    ];
    for (who, groups, name, path) in pages {
        harness.issuer.set_groups(groups);
        let token = session_token(&harness.sign_in().await).expect("sign-in sets a session");
        let body = body_text(harness.get(path, Some(&token)).await)
            .await
            .replace("href=\"/pkg/crabbage.css\"", "href=\"crabbage.css\"");
        std::fs::write(dir.join(format!("{who}-{name}.html")), body).expect("page is written");
    }
}

/// A recipe with four lines and three steps, owned by the member who signs in first.
fn sample_recipe() -> RecipeInput {
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
    }
}
