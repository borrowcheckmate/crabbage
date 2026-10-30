//! The catalog endpoints: admins manage ingredients, aliases, store sections and the review queue,
//! and every other signed-in user is refused.

use axum::http::StatusCode;
use crabbage_store::catalog::ingredients::{self, list};
use crabbage_store::catalog::sections;
use sqlx::sqlite::SqlitePool;

use crate::common::{Harness, body_text, session_token};

const ADMIN: &str = "crabbage-admin";

async fn token_for(harness: &Harness, groups: &[&str]) -> String {
    harness.issuer.set_groups(groups);
    session_token(&harness.sign_in().await).expect("sign-in sets a session")
}

/// Posts to `endpoint` and returns the status and the body.
async fn call(harness: &Harness, token: &str, endpoint: &str, form: &str) -> (StatusCode, String) {
    let response = harness
        .post_form(&format!("/api/{endpoint}"), Some(token), form)
        .await;
    let status = response.status();
    (status, body_text(response).await)
}

async fn ok(harness: &Harness, token: &str, endpoint: &str, form: &str) -> String {
    let (status, body) = call(harness, token, endpoint, form).await;
    assert_eq!(status, StatusCode::OK, "{endpoint}: {body}");
    body
}

async fn id_of(pool: &SqlitePool, name: &str) -> i64 {
    sqlx::query_scalar("SELECT id FROM ingredients WHERE name = ?")
        .bind(name)
        .fetch_one(pool)
        .await
        .expect("the ingredient exists")
}

async fn ingredient_names(pool: &SqlitePool) -> Vec<String> {
    list(pool)
        .await
        .expect("ingredients read")
        .into_iter()
        .map(|ingredient| ingredient.name)
        .collect()
}

#[tokio::test]
async fn admin_creates_renames_and_deletes_an_ingredient() {
    let harness = Harness::start("catalog-crud").await;
    let token = token_for(&harness, &[ADMIN]).await;

    ok(&harness, &token, "create_ingredient", "name=Tomato").await;
    let id = id_of(&harness.pool, "Tomato").await;

    ok(
        &harness,
        &token,
        "rename_ingredient",
        &format!("id={id}&name=Cherry+tomato"),
    )
    .await;
    assert_eq!(ingredient_names(&harness.pool).await, ["Cherry tomato"]);

    ok(&harness, &token, "delete_ingredient", &format!("id={id}")).await;
    assert_eq!(ingredient_names(&harness.pool).await, Vec::<String>::new());

    let (status, _) = call(&harness, &token, "delete_ingredient", &format!("id={id}")).await;
    assert_ne!(
        status,
        StatusCode::OK,
        "a missing ingredient is not deleted twice"
    );
}

/// Every write endpoint with a form that would change something, and the admin list.
fn write_forms(tomato: i64, onion: i64, produce: i64) -> Vec<(&'static str, String)> {
    vec![
        ("create_ingredient", "name=Onion".to_owned()),
        ("rename_ingredient", format!("id={tomato}&name=Garlic")),
        ("delete_ingredient", format!("id={tomato}")),
        (
            "add_alias",
            format!("ingredient_id={tomato}&alias=Pomodoro"),
        ),
        (
            "remove_alias",
            format!("ingredient_id={tomato}&alias=Pomodoro"),
        ),
        (
            "set_ingredient_section",
            format!("id={tomato}&section_id={produce}"),
        ),
        ("merge_ingredients", format!("from={tomato}&into={onion}")),
        ("approve_ingredient", format!("id={tomato}")),
        ("create_section", "name=Dairy".to_owned()),
        ("rename_section", format!("id={produce}&name=Dairy")),
        ("delete_section", format!("id={produce}")),
        ("move_section", format!("id={produce}&position=0")),
        ("list_ingredients", String::new()),
    ]
}

#[tokio::test]
async fn members_are_refused_for_every_write() {
    let harness = Harness::start("catalog-member").await;
    ingredients::create(&harness.pool, "Tomato").await.unwrap();
    let tomato = id_of(&harness.pool, "Tomato").await;
    let onion = ingredients::create(&harness.pool, "Onion").await.unwrap();
    sqlx::query("INSERT INTO ingredients (name, pending_review) VALUES ('Okra', 1)")
        .execute(&harness.pool)
        .await
        .unwrap();
    let produce = sections::create(&harness.pool, "Produce").await.unwrap();
    let token = token_for(&harness, &[]).await;

    for (endpoint, form) in write_forms(tomato, onion, produce) {
        let (status, body) = call(&harness, &token, endpoint, &form).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{endpoint}: {body}");
        assert!(body.contains("not allowed"), "{endpoint}: {body}");
    }

    assert_eq!(
        ingredient_names(&harness.pool).await,
        ["Okra", "Onion", "Tomato"]
    );
    let names: Vec<String> = sections::list(&harness.pool)
        .await
        .unwrap()
        .into_iter()
        .map(|section| section.name)
        .collect();
    assert_eq!(names, ["Produce"]);
    let pending: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM ingredients WHERE pending_review = 1")
            .fetch_one(&harness.pool)
            .await
            .unwrap();
    assert_eq!(pending, 1);
}

#[tokio::test]
async fn aliases_are_case_insensitively_unique_through_the_api() {
    let harness = Harness::start("catalog-aliases").await;
    let token = token_for(&harness, &[ADMIN]).await;
    ok(&harness, &token, "create_ingredient", "name=Tomato").await;
    ok(&harness, &token, "create_ingredient", "name=Potato").await;
    let tomato = id_of(&harness.pool, "Tomato").await;
    let potato = id_of(&harness.pool, "Potato").await;

    ok(
        &harness,
        &token,
        "add_alias",
        &format!("ingredient_id={tomato}&alias=Pomodoro"),
    )
    .await;
    for (id, alias) in [
        (potato, "pomodoro"),
        (tomato, "POMODORO"),
        (tomato, "potato"),
        (potato, "tomato"),
    ] {
        let (status, body) = call(
            &harness,
            &token,
            "add_alias",
            &format!("ingredient_id={id}&alias={alias}"),
        )
        .await;
        assert_ne!(status, StatusCode::OK, "{alias} is accepted: {body}");
        assert!(body.contains("already in use"), "{alias}: {body}");
    }
    let (status, body) = call(&harness, &token, "create_ingredient", "name=pomodoro").await;
    assert_ne!(
        status,
        StatusCode::OK,
        "a name equal to an alias is accepted: {body}"
    );
}

#[tokio::test]
async fn sections_are_created_and_reordered_through_the_api() {
    let harness = Harness::start("catalog-sections").await;
    let token = token_for(&harness, &[ADMIN]).await;
    for name in ["Dairy", "Bakery", "Frozen"] {
        ok(&harness, &token, "create_section", &format!("name={name}")).await;
    }
    let frozen: i64 = sqlx::query_scalar("SELECT id FROM store_sections WHERE name = 'Frozen'")
        .fetch_one(&harness.pool)
        .await
        .unwrap();

    ok(
        &harness,
        &token,
        "move_section",
        &format!("id={frozen}&position=0"),
    )
    .await;

    let names: Vec<String> = sections::list(&harness.pool)
        .await
        .unwrap()
        .into_iter()
        .map(|section| section.name)
        .collect();
    assert_eq!(names, ["Frozen", "Dairy", "Bakery"]);
}

#[tokio::test]
async fn the_review_queue_lists_pending_ingredients_and_approve_clears_them() {
    let harness = Harness::start("catalog-review").await;
    let token = token_for(&harness, &[ADMIN]).await;
    sqlx::query("INSERT INTO ingredients (name, pending_review) VALUES ('Okra', 1)")
        .execute(&harness.pool)
        .await
        .unwrap();

    let body = ok(&harness, &token, "list_pending_ingredients", "").await;
    assert!(body.contains("Okra"), "{body}");

    let id = id_of(&harness.pool, "Okra").await;
    ok(&harness, &token, "approve_ingredient", &format!("id={id}")).await;

    assert_eq!(
        ingredients::list_pending(&harness.pool)
            .await
            .unwrap()
            .len(),
        0
    );
    let body = ok(&harness, &token, "list_pending_ingredients", "").await;
    assert!(!body.contains("Okra"), "{body}");
}

#[tokio::test]
async fn merge_repoints_recipe_lines_and_moves_aliases_through_the_api() {
    let harness = Harness::start("catalog-merge").await;
    let token = token_for(&harness, &[ADMIN]).await;
    ok(&harness, &token, "create_ingredient", "name=Scallion").await;
    ok(&harness, &token, "create_ingredient", "name=Spring+onion").await;
    let scallion = id_of(&harness.pool, "Scallion").await;
    let spring = id_of(&harness.pool, "Spring onion").await;
    ok(
        &harness,
        &token,
        "add_alias",
        &format!("ingredient_id={scallion}&alias=Green+onion"),
    )
    .await;
    insert_recipe_line(&harness, scallion).await;

    ok(
        &harness,
        &token,
        "merge_ingredients",
        &format!("from={scallion}&into={spring}"),
    )
    .await;

    let lines: Vec<i64> = sqlx::query_scalar("SELECT ingredient_id FROM recipe_lines")
        .fetch_all(&harness.pool)
        .await
        .unwrap();
    assert_eq!(lines, [spring]);
    let ingredients = list(&harness.pool).await.unwrap();
    assert_eq!(ingredients.len(), 1);
    assert_eq!(ingredients[0].aliases, ["Green onion", "Scallion"]);
}

#[expect(
    clippy::unwrap_used,
    reason = "test helpers stop the test on setup failure"
)]
async fn insert_recipe_line(harness: &Harness, ingredient: i64) {
    let user = harness.user_id().await;
    let recipe: i64 = sqlx::query_scalar(
        "INSERT INTO recipes (owner_id, title, servings, created_at) VALUES (?, 'Soup', 2, 0) RETURNING id",
    )
    .bind(user)
    .fetch_one(&harness.pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO recipe_lines (recipe_id, position, ingredient_id) VALUES (?, 0, ?)")
        .bind(recipe)
        .bind(ingredient)
        .execute(&harness.pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn validation_and_in_use_errors_are_reported_in_words() {
    let harness = Harness::start("catalog-errors").await;
    let token = token_for(&harness, &[ADMIN]).await;
    let (status, body) = call(&harness, &token, "create_ingredient", "name=+++").await;
    assert_ne!(status, StatusCode::OK);
    assert!(body.contains("Enter a name."), "{body}");

    ok(&harness, &token, "create_ingredient", "name=Tomato").await;
    let tomato = id_of(&harness.pool, "Tomato").await;
    insert_recipe_line(&harness, tomato).await;
    let (_, body) = call(
        &harness,
        &token,
        "delete_ingredient",
        &format!("id={tomato}"),
    )
    .await;
    assert!(body.contains("cannot be deleted"), "{body}");

    let (_, body) = call(
        &harness,
        &token,
        "set_ingredient_section",
        &format!("id={tomato}&section_id=abc"),
    )
    .await;
    assert!(body.contains("Choose a store section"), "{body}");
}

#[tokio::test]
async fn ingredients_are_filed_and_their_aliases_removed_through_the_api() {
    let harness = Harness::start("catalog-filing").await;
    let token = token_for(&harness, &[ADMIN]).await;
    ok(&harness, &token, "create_section", "name=Dairy").await;
    ok(&harness, &token, "create_ingredient", "name=Milk").await;
    let dairy = section_id(&harness.pool, "Dairy").await;
    let milk = id_of(&harness.pool, "Milk").await;

    let filed = format!("id={milk}&section_id={dairy}");
    ok(&harness, &token, "set_ingredient_section", &filed).await;
    assert_eq!(
        list(&harness.pool).await.unwrap()[0].store_section_id,
        Some(dairy)
    );
    let cleared = format!("id={milk}&section_id=");
    ok(&harness, &token, "set_ingredient_section", &cleared).await;
    assert_eq!(list(&harness.pool).await.unwrap()[0].store_section_id, None);

    let alias = format!("ingredient_id={milk}&alias=Whole+milk");
    ok(&harness, &token, "add_alias", &alias).await;
    ok(&harness, &token, "remove_alias", &alias).await;
    assert_eq!(list(&harness.pool).await.unwrap()[0].aliases.len(), 0);

    let (_, body) = call(
        &harness,
        &token,
        "add_alias",
        "ingredient_id=999&alias=Ghost",
    )
    .await;
    assert!(body.contains("does not exist"), "{body}");
    let (_, body) = call(&harness, &token, "rename_ingredient", "id=999&name=Ghost").await;
    assert!(body.contains("does not exist"), "{body}");
}

#[tokio::test]
async fn sections_are_renamed_and_deleted_through_the_api() {
    let harness = Harness::start("catalog-section-edit").await;
    let token = token_for(&harness, &[ADMIN]).await;
    ok(&harness, &token, "create_section", "name=Dairy").await;
    let dairy = section_id(&harness.pool, "Dairy").await;

    ok(
        &harness,
        &token,
        "rename_section",
        &format!("id={dairy}&name=Chilled"),
    )
    .await;
    ok(&harness, &token, "delete_section", &format!("id={dairy}")).await;
    let (status, body) = call(&harness, &token, "delete_section", &format!("id={dairy}")).await;
    assert_ne!(status, StatusCode::OK, "{body}");
    assert!(body.contains("does not exist"), "{body}");
}

async fn section_id(pool: &SqlitePool, name: &str) -> i64 {
    sqlx::query_scalar("SELECT id FROM store_sections WHERE name = ?")
        .bind(name)
        .fetch_one(pool)
        .await
        .expect("the section exists")
}
