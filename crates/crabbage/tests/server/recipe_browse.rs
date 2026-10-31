//! Browsing recipes on a phone: the search by name or ingredient combined with the filters, the
//! servings stepper of a recipe page, and the viewport the phone layout relies on.

use axum::http::StatusCode;
use crabbage_store::catalog::categories;
use crabbage_store::recipes::{LineInput, RecipeInput, create as create_recipe, set_favourite};

use crate::common::{Harness, body_text, sign_in_as};
use crate::recipes::create;

const OWNER: &str = "owner";

async fn page(harness: &Harness, token: &str, path: &str) -> String {
    let response = harness.get(path, Some(token)).await;
    assert_eq!(response.status(), StatusCode::OK, "{path}");
    body_text(response).await
}

/// Stores a recipe owned by the owner with one 100 g line per ingredient, filed under
/// `category_ids`, and returns its id.
#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
async fn stored(
    harness: &Harness,
    title: &str,
    ingredients: &[&str],
    category_ids: Vec<i64>,
) -> i64 {
    let owner_id: i64 = sqlx::query_scalar("SELECT id FROM users WHERE subject = ?")
        .bind(OWNER)
        .fetch_one(&harness.pool)
        .await
        .expect("the user exists");
    let input = RecipeInput {
        title: title.to_owned(),
        servings: 4,
        lines: ingredients
            .iter()
            .copied()
            .map(|name| LineInput {
                amount: Some(100.0),
                unit: Some("g".to_owned()),
                ingredient: Some(name.to_owned()),
                note: None,
            })
            .collect(),
        steps: Vec::new(),
        category_ids,
    };
    create_recipe(&harness.pool, owner_id, &input, 0)
        .await
        .expect("recipe is stored")
}

#[tokio::test]
async fn search_finds_recipes_by_title_or_ingredient_ignoring_case() {
    let harness = Harness::start("browse-search").await;
    let member = sign_in_as(&harness, OWNER, &[]).await;
    stored(&harness, "Tomato soup", &["Onion"], vec![]).await;
    stored(&harness, "Pancakes", &["Flour", "Egg"], vec![]).await;
    stored(&harness, "Roast", &["Pork"], vec![]).await;

    let body = page(&harness, &member, "/recipes?q=TOMATO").await;
    assert!(body.contains("Tomato soup</a>"), "{body}");
    assert!(!body.contains("Pancakes</a>"), "{body}");
    assert!(body.contains("value=\"TOMATO\""), "{body}");
    assert!(body.contains("name=\"q\""), "{body}");

    let body = page(&harness, &member, "/recipes?q=flour").await;
    assert!(body.contains("Pancakes</a>"), "{body}");
    assert!(!body.contains("Roast</a>"), "{body}");

    let body = page(&harness, &member, "/recipes?q=pork").await;
    assert!(body.contains("Roast</a>"), "{body}");
    assert!(!body.contains("Pancakes</a>"), "{body}");

    let body = page(&harness, &member, "/recipes?q=%20%20").await;
    for title in ["Tomato soup</a>", "Pancakes</a>", "Roast</a>"] {
        assert!(body.contains(title), "{title}: {body}");
    }
}

#[tokio::test]
async fn a_search_without_a_match_names_the_search() {
    let harness = Harness::start("browse-no-match").await;
    let member = sign_in_as(&harness, OWNER, &[]).await;
    stored(&harness, "Pancakes", &["Flour"], vec![]).await;

    let body = page(&harness, &member, "/recipes?q=zebra").await;
    assert!(body.contains("No recipes match"), "{body}");
    assert!(body.contains("zebra"), "{body}");
    assert!(!body.contains("Pancakes</a>"), "{body}");
}

#[tokio::test]
async fn the_search_combines_with_the_filters_and_the_filters_keep_it() {
    let harness = Harness::start("browse-filters").await;
    let member = sign_in_as(&harness, OWNER, &[]).await;
    let breakfast = categories::create(&harness.pool, "Breakfast")
        .await
        .unwrap();
    let dinner = categories::create(&harness.pool, "Dinner").await.unwrap();
    let pancakes = stored(&harness, "Pancakes", &["Flour"], vec![breakfast]).await;
    stored(&harness, "Waffles", &["Flour"], vec![dinner]).await;
    stored(&harness, "Toast", &["Bread"], vec![breakfast]).await;
    let owner_id: i64 = sqlx::query_scalar("SELECT id FROM users WHERE subject = ?")
        .bind(OWNER)
        .fetch_one(&harness.pool)
        .await
        .unwrap();
    set_favourite(&harness.pool, pancakes, owner_id, true)
        .await
        .unwrap();

    let body = page(&harness, &member, "/recipes?q=flour").await;
    assert!(body.contains("Pancakes</a>"), "{body}");
    assert!(body.contains("Waffles</a>"), "{body}");
    assert!(!body.contains("Toast</a>"), "{body}");
    assert!(
        body.contains(&format!(
            "href=\"/recipes?category={breakfast}&amp;q=flour\""
        )),
        "{body}"
    );
    assert!(body.contains("href=\"/recipes?q=flour\""), "{body}");
    assert!(
        body.contains("href=\"/recipes?favourites=1&amp;q=flour\""),
        "{body}"
    );

    let body = page(
        &harness,
        &member,
        &format!("/recipes?category={breakfast}&q=flour"),
    )
    .await;
    assert!(body.contains("Pancakes</a>"), "{body}");
    assert!(!body.contains("Waffles</a>"), "{body}");
    assert!(
        body.contains(&format!("name=\"category\" value=\"{breakfast}\"")),
        "{body}"
    );

    let body = page(&harness, &member, "/recipes?favourites=1&q=flour").await;
    assert!(body.contains("Pancakes</a>"), "{body}");
    assert!(!body.contains("Waffles</a>"), "{body}");
    assert!(body.contains("name=\"favourites\" value=\"1\""), "{body}");

    let body = page(&harness, &member, "/recipes?favourites=1&q=bread").await;
    assert!(body.contains("No recipes match"), "{body}");
    assert!(!body.contains("You have no favourites yet."), "{body}");
}

#[tokio::test]
async fn a_search_text_is_encoded_in_the_filter_links() {
    let harness = Harness::start("browse-encoding").await;
    let member = sign_in_as(&harness, OWNER, &[]).await;
    stored(&harness, "Brown rice bowl", &["Rice"], vec![]).await;

    let body = page(&harness, &member, "/recipes?q=brown%20rice").await;
    assert!(body.contains("Brown rice bowl</a>"), "{body}");
    assert!(body.contains("href=\"/recipes?q=brown%20rice\""), "{body}");
}

#[tokio::test]
async fn the_servings_value_rescales_the_amounts() {
    let harness = Harness::start("browse-servings").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &owner, "Pancakes").await;

    let body = page(&harness, &owner, &format!("/recipes/{id}")).await;
    assert!(body.contains("<li>250 g Flour, sifted</li>"), "{body}");
    assert!(body.contains("4 servings"), "{body}");
    assert!(!body.contains(">Reset</a>"), "{body}");

    let body = page(&harness, &owner, &format!("/recipes/{id}?servings=8")).await;
    assert!(body.contains("<li>500 g Flour, sifted</li>"), "{body}");
    assert!(body.contains("<li>4 Egg</li>"), "{body}");
    assert!(body.contains("<li>Salt, to taste</li>"), "{body}");
    assert!(body.contains("8 servings"), "{body}");
    assert!(
        body.contains(&format!("href=\"/recipes/{id}\">Reset</a>")),
        "{body}"
    );
}

#[tokio::test]
async fn invalid_servings_values_fall_back_to_the_recipes_own() {
    let harness = Harness::start("browse-servings-invalid").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &owner, "Pancakes").await;

    for query in ["0", "abc", "1000", "-3", ""] {
        let body = page(&harness, &owner, &format!("/recipes/{id}?servings={query}")).await;
        assert!(
            body.contains("<li>250 g Flour, sifted</li>"),
            "{query}: {body}"
        );
        assert!(body.contains("4 servings"), "{query}: {body}");
        assert!(!body.contains(">Reset</a>"), "{query}: {body}");
    }
}

#[tokio::test]
async fn the_stepper_links_change_the_servings_and_stop_at_the_limits() {
    let harness = Harness::start("browse-stepper").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &owner, "Pancakes").await;

    let body = page(&harness, &owner, &format!("/recipes/{id}")).await;
    assert!(
        body.contains(&format!("href=\"/recipes/{id}?servings=3\"")),
        "{body}"
    );
    assert!(
        body.contains(&format!("href=\"/recipes/{id}?servings=5\"")),
        "{body}"
    );
    assert!(body.contains("aria-label=\"Fewer servings\""), "{body}");
    assert!(body.contains("aria-label=\"More servings\""), "{body}");

    let body = page(&harness, &owner, &format!("/recipes/{id}?servings=1")).await;
    assert!(body.contains("1 serving</span>"), "{body}");
    assert!(
        body.contains(&format!("href=\"/recipes/{id}?servings=2\"")),
        "{body}"
    );
    assert!(
        !body.contains(&format!("href=\"/recipes/{id}?servings=0\"")),
        "{body}"
    );
    assert!(!body.contains("aria-label=\"Fewer servings\""), "{body}");

    let body = page(&harness, &owner, &format!("/recipes/{id}?servings=99")).await;
    assert!(
        body.contains(&format!("href=\"/recipes/{id}?servings=98\"")),
        "{body}"
    );
    assert!(
        !body.contains(&format!("href=\"/recipes/{id}?servings=100\"")),
        "{body}"
    );
    assert!(!body.contains("aria-label=\"More servings\""), "{body}");
}

#[tokio::test]
async fn the_list_and_the_recipe_page_declare_a_phone_viewport() {
    let harness = Harness::start("browse-viewport").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &owner, "Pancakes").await;
    for path in ["/recipes".to_owned(), format!("/recipes/{id}")] {
        let body = page(&harness, &owner, &path).await;
        assert!(
            body.contains("name=\"viewport\" content=\"width=device-width"),
            "{path}: {body}"
        );
    }
}
