//! The recipe endpoints: every member creates and views recipes, the owner and their editors change
//! them, the owner and admins delete them and manage the editors, and invalid input is refused in
//! words.

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::sqlite::SqlitePool;

use crate::common::{Harness, body_text, sign_in_as};

const ADMIN: &str = "crabbage-admin";
const OWNER: &str = "owner";
const OTHER: &str = "other";
const EDITOR: &str = "editor";

/// Posts the form pairs to `endpoint` with `token`, and returns the status and the body.
async fn call(
    harness: &Harness,
    token: &str,
    endpoint: &str,
    pairs: &[(&str, &str)],
) -> (StatusCode, String) {
    let body = url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs(pairs)
        .finish();
    let response = harness
        .post_form(&format!("/api/{endpoint}"), Some(token), &body)
        .await;
    let status = response.status();
    (status, body_text(response).await)
}

/// A recipe with three lines and three steps, as the form sends it.
pub(crate) fn draft(title: &str) -> String {
    json!({
        "title": title,
        "servings": "4",
        "lines": [
            {"amount": "250", "unit": "g", "ingredient": "Flour", "note": "sifted"},
            {"amount": "2", "unit": "piece", "ingredient": "Egg", "note": ""},
            {"amount": "", "unit": "to taste", "ingredient": "Salt", "note": ""}
        ],
        "steps": [
            {"text": "Whisk the eggs", "minutes": "2"},
            {"text": "Stir in the flour", "minutes": ""},
            {"text": "Fry until golden", "minutes": "4"}
        ]
    })
    .to_string()
}

/// Creates a recipe as `token` and returns its id.
pub(crate) async fn create(harness: &Harness, token: &str, title: &str) -> i64 {
    let (status, body) = call(harness, token, "create_recipe", &[("draft", &draft(title))]).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body.trim().parse().expect("the id is a number")
}

/// The recipe `id` as the page record, as `token` sees it.
async fn view(harness: &Harness, token: &str, id: i64) -> Value {
    let (status, body) = call(harness, token, "get_recipe", &[("id", &id.to_string())]).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    serde_json::from_str(&body).expect("the record is JSON")
}

async fn user_id(pool: &SqlitePool, subject: &str) -> i64 {
    sqlx::query_scalar("SELECT id FROM users WHERE subject = ?")
        .bind(subject)
        .fetch_one(pool)
        .await
        .expect("the user exists")
}

async fn user_name(pool: &SqlitePool, subject: &str) -> String {
    sqlx::query_scalar("SELECT name FROM users WHERE subject = ?")
        .bind(subject)
        .fetch_one(pool)
        .await
        .expect("the user exists")
}

async fn recipe_count(pool: &SqlitePool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM recipes")
        .fetch_one(pool)
        .await
        .expect("count reads")
}

#[tokio::test]
async fn a_member_creates_a_recipe_and_gets_it_back() {
    let harness = Harness::start("recipes-create").await;
    let token = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &token, "Pancakes").await;

    let recipe = view(&harness, &token, id).await;
    assert_eq!(recipe["title"], "Pancakes");
    assert_eq!(recipe["servings"], 4);
    assert_eq!(recipe["owner_name"], user_name(&harness.pool, OWNER).await);
    assert_eq!(recipe["lines"].as_array().map(Vec::len), Some(3));
    assert_eq!(recipe["lines"][0]["amount"], 250.0);
    assert_eq!(recipe["lines"][0]["unit"], "g");
    assert_eq!(recipe["lines"][0]["ingredient"], "Flour");
    assert_eq!(recipe["lines"][0]["note"], "sifted");
    assert_eq!(recipe["lines"][2]["amount"], Value::Null);
    assert_eq!(recipe["steps"][0]["duration_seconds"], 120);
    assert_eq!(recipe["steps"][1]["duration_seconds"], Value::Null);
    assert_eq!(recipe["can_edit"], true);
    assert_eq!(recipe["can_delete"], true);
    assert_eq!(recipe["can_manage_editors"], true);

    let pending: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM ingredients WHERE pending_review = 1")
            .fetch_one(&harness.pool)
            .await
            .expect("count reads");
    assert_eq!(pending, 3, "the three new ingredients wait for review");
}

#[tokio::test]
async fn another_member_views_but_cannot_change_a_recipe() {
    let harness = Harness::start("recipes-other").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &owner, "Pancakes").await;
    let other = sign_in_as(&harness, OTHER, &[]).await;
    let other_id = user_id(&harness.pool, OTHER).await;

    let recipe = view(&harness, &other, id).await;
    assert_eq!(recipe["title"], "Pancakes");
    assert_eq!(recipe["can_edit"], false);
    assert_eq!(recipe["can_delete"], false);
    assert_eq!(recipe["can_manage_editors"], false);

    let changed = draft("Hijacked");
    for (endpoint, pairs) in [
        (
            "update_recipe",
            vec![("id", id.to_string()), ("draft", changed.clone())],
        ),
        ("delete_recipe", vec![("id", id.to_string())]),
        (
            "add_recipe_editor",
            vec![
                ("recipe_id", id.to_string()),
                ("user_id", other_id.to_string()),
            ],
        ),
    ] {
        let pairs: Vec<(&str, &str)> = pairs.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let (status, body) = call(&harness, &other, endpoint, &pairs).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{endpoint}: {body}");
        assert!(body.contains("not allowed"), "{endpoint}: {body}");
    }
    assert_eq!(view(&harness, &owner, id).await["title"], "Pancakes");
    assert_eq!(recipe_count(&harness.pool).await, 1);
}

#[tokio::test]
async fn an_editor_changes_a_recipe_but_cannot_delete_or_manage_editors() {
    let harness = Harness::start("recipes-editor").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &owner, "Pancakes").await;
    let editor = sign_in_as(&harness, EDITOR, &[]).await;
    let editor_id = user_id(&harness.pool, EDITOR).await;

    let (status, body) = call(
        &harness,
        &owner,
        "add_recipe_editor",
        &[
            ("recipe_id", &id.to_string()),
            ("user_id", &editor_id.to_string()),
        ],
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(
        body.contains(&user_name(&harness.pool, EDITOR).await),
        "{body}"
    );
    assert_eq!(view(&harness, &editor, id).await["can_edit"], true);

    let (status, body) = call(
        &harness,
        &editor,
        "update_recipe",
        &[("id", &id.to_string()), ("draft", &draft("Crêpes"))],
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(view(&harness, &owner, id).await["title"], "Crêpes");

    let refused = [
        ("delete_recipe", vec![("id", id.to_string())]),
        (
            "add_recipe_editor",
            vec![
                ("recipe_id", id.to_string()),
                ("user_id", editor_id.to_string()),
            ],
        ),
        (
            "remove_recipe_editor",
            vec![
                ("recipe_id", id.to_string()),
                ("user_id", editor_id.to_string()),
            ],
        ),
    ];
    for (endpoint, pairs) in refused {
        let pairs: Vec<(&str, &str)> = pairs.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let (status, body) = call(&harness, &editor, endpoint, &pairs).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{endpoint}: {body}");
    }
    assert_eq!(recipe_count(&harness.pool).await, 1);
}

#[tokio::test]
async fn removing_an_editor_takes_away_the_right_to_change() {
    let harness = Harness::start("recipes-editor-removed").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &owner, "Pancakes").await;
    let editor = sign_in_as(&harness, EDITOR, &[]).await;
    let editor_id = user_id(&harness.pool, EDITOR).await;
    let (status, body) = call(
        &harness,
        &owner,
        "add_recipe_editor",
        &[
            ("recipe_id", &id.to_string()),
            ("user_id", &editor_id.to_string()),
        ],
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let (status, body) = call(
        &harness,
        &owner,
        "remove_recipe_editor",
        &[
            ("recipe_id", &id.to_string()),
            ("user_id", &editor_id.to_string()),
        ],
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, _) = call(
        &harness,
        &editor,
        "update_recipe",
        &[("id", &id.to_string()), ("draft", &draft("Again"))],
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "a removed editor is refused");
}

#[tokio::test]
async fn an_admin_changes_and_deletes_anyones_recipe() {
    let harness = Harness::start("recipes-admin").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &owner, "Pancakes").await;
    let admin = sign_in_as(&harness, "admin", &[ADMIN]).await;

    let recipe = view(&harness, &admin, id).await;
    assert_eq!(recipe["can_delete"], true);

    let (status, body) = call(
        &harness,
        &admin,
        "update_recipe",
        &[
            ("id", &id.to_string()),
            ("draft", &draft("Admin's pancakes")),
        ],
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = call(
        &harness,
        &admin,
        "delete_recipe",
        &[("id", &id.to_string())],
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(recipe_count(&harness.pool).await, 0);
    let (_, body) = call(&harness, &admin, "get_recipe", &[("id", &id.to_string())]).await;
    assert!(body.contains("does not exist"), "{body}");
}

#[tokio::test]
async fn invalid_input_is_refused_with_a_reason() {
    let harness = Harness::start("recipes-invalid").await;
    let token = sign_in_as(&harness, OWNER, &[]).await;
    let cases = [
        (json!({"title": "  ", "servings": "2"}), "Enter a title."),
        (
            json!({"title": "Soup", "servings": "0"}),
            "Servings must be a whole number",
        ),
        (
            json!({"title": "Soup", "servings": "two"}),
            "Servings must be a whole number",
        ),
        (
            json!({"title": "Soup", "servings": "2", "lines": [{"amount": "-1", "unit": "g", "ingredient": "Salt", "note": ""}]}),
            "An amount must be a number",
        ),
        (
            json!({"title": "Soup", "servings": "2", "lines": [{"amount": "1", "unit": "cup", "ingredient": "Milk", "note": ""}]}),
            "Choose a unit from the list.",
        ),
        (
            json!({"title": "Soup", "servings": "2", "steps": [{"text": " ", "minutes": "3"}]}),
            "Enter the text of the step.",
        ),
        (
            json!({"title": "Soup", "servings": "2", "steps": [{"text": "Stir", "minutes": "0"}]}),
            "Minutes must be a whole number",
        ),
    ];
    for (draft, reason) in cases {
        let (status, body) = call(
            &harness,
            &token,
            "create_recipe",
            &[("draft", &draft.to_string())],
        )
        .await;
        assert_ne!(status, StatusCode::OK, "{draft}: {body}");
        assert!(body.contains(reason), "{reason}: {body}");
    }
    assert_eq!(recipe_count(&harness.pool).await, 0);
}

#[tokio::test]
async fn invalid_input_is_refused_in_dutch_when_the_app_speaks_dutch() {
    let harness = Harness::start_with_language("recipes-nl", Some("nl")).await;
    let token = sign_in_as(&harness, OWNER, &[]).await;
    let (status, body) = call(
        &harness,
        &token,
        "create_recipe",
        &[("draft", &json!({"title": "", "servings": "2"}).to_string())],
    )
    .await;
    assert_ne!(status, StatusCode::OK, "{body}");
    assert!(body.contains("Vul een titel in."), "{body}");
}

#[tokio::test]
async fn every_member_sees_the_list_and_the_names_of_users_and_ingredients() {
    let harness = Harness::start("recipes-list").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    create(&harness, &owner, "Soup").await;
    create(&harness, &owner, "Bread").await;
    let other = sign_in_as(&harness, OTHER, &[]).await;

    let (status, body) = call(
        &harness,
        &other,
        "list_recipes",
        &[("favourites_only", "false")],
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let titles: Vec<String> = serde_json::from_str::<Vec<Value>>(&body)
        .expect("a list")
        .into_iter()
        .map(|row| row["title"].as_str().unwrap_or_default().to_owned())
        .collect();
    assert_eq!(titles, ["Bread", "Soup"]);

    let (_, body) = call(&harness, &other, "list_users", &[]).await;
    assert!(
        body.contains(&user_name(&harness.pool, OWNER).await),
        "{body}"
    );
    let (_, body) = call(&harness, &other, "ingredient_names", &[]).await;
    assert!(body.contains("Flour"), "{body}");
}

/// The draft of `title` filed under `categories`, as the form sends it.
fn filed(title: &str, categories: &[i64]) -> String {
    let mut value: Value = serde_json::from_str(&draft(title)).expect("draft is JSON");
    value
        .as_object_mut()
        .expect("draft is an object")
        .insert("categories".to_owned(), json!(categories));
    value.to_string()
}

/// The names of the categories of a recipe record, in order.
fn category_names(recipe: &Value) -> Vec<String> {
    recipe["categories"]
        .as_array()
        .expect("categories are a list")
        .iter()
        .map(|category| category["name"].as_str().expect("a name").to_owned())
        .collect()
}

#[tokio::test]
async fn editors_file_a_recipe_under_any_number_of_categories() {
    let harness = Harness::start("recipes-categories").await;
    let breakfast = crabbage_store::catalog::categories::create(&harness.pool, "Breakfast")
        .await
        .unwrap();
    let dinner = crabbage_store::catalog::categories::create(&harness.pool, "Dinner")
        .await
        .unwrap();
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let editor = sign_in_as(&harness, EDITOR, &[]).await;
    let editor_id = user_id(&harness.pool, EDITOR).await;
    let id = {
        let (status, body) = call(
            &harness,
            &owner,
            "create_recipe",
            &[("draft", &filed("Pancakes", &[breakfast, dinner, 999]))],
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{body}");
        body.trim().parse::<i64>().expect("the id is a number")
    };
    assert_eq!(
        category_names(&view(&harness, &owner, id).await),
        ["Breakfast", "Dinner"]
    );

    crabbage_store::recipes::add_editor(&harness.pool, id, editor_id)
        .await
        .unwrap();
    let (status, body) = call(
        &harness,
        &editor,
        "update_recipe",
        &[
            ("id", &id.to_string()),
            ("draft", &filed("Pancakes", &[dinner])),
        ],
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        category_names(&view(&harness, &owner, id).await),
        ["Dinner"]
    );

    let (status, body) = call(
        &harness,
        &editor,
        "update_recipe",
        &[("id", &id.to_string()), ("draft", &filed("Pancakes", &[]))],
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        category_names(&view(&harness, &owner, id).await),
        Vec::<String>::new()
    );
}
