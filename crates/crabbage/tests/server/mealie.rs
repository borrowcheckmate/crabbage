use std::time::Duration;

use axum::http::StatusCode;
use crabbage_store::recipes::imported_mealie_ids;
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::common::{Harness, body_text, sign_in_as};

pub(crate) async fn mealie_server(unauthorized: bool) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let address = listener.local_addr().expect("address");
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(answer(stream, unauthorized));
        }
    });
    format!("http://{address}")
}

async fn answer(mut stream: TcpStream, unauthorized: bool) {
    let mut request = [0_u8; 4096];
    let length = stream.read(&mut request).await.unwrap_or_default();
    let request = String::from_utf8_lossy(request.get(..length).unwrap_or_default());
    let target = request.split_whitespace().nth(1).unwrap_or_default();
    let (status, body) = if unauthorized {
        (401, "{}".to_owned())
    } else if target.starts_with("/api/recipes/citrus-cakes") {
        let mut recipe: serde_json::Value = serde_json::from_str(include_str!(
            "../../../crabbage-import/tests/fixtures/mealie/recipe.json"
        ))
        .unwrap_or_default();
        if let Some(lines) = recipe
            .get_mut("recipeIngredient")
            .and_then(serde_json::Value::as_array_mut)
        {
            lines.push(json!({"note":"salt to taste","originalText":"salt to taste"}));
        }
        (200, recipe.to_string())
    } else if target.starts_with("/api/recipes/broken") {
        (404, "{}".to_owned())
    } else {
        let items = if target.starts_with("/api/recipes?") {
            json!([
                {"id":"recipe-42","slug":"citrus-cakes","name":"Citrus Cakes"},
                {"id":"recipe-broken","slug":"broken","name":"Broken Tart"}
            ])
        } else if target.starts_with("/api/foods?") {
            json!([{"id":"food-flour","name":"Flour","aliases":[{"name":"Plain Flour"}]}])
        } else if target.starts_with("/api/organizers/categories?") {
            json!([{"id":"cat-baking","name":"Baking","slug":"baking"}])
        } else if target.starts_with("/api/organizers/tags?") {
            json!([{"id":"tag-citrus","name":"Citrus","slug":"citrus"}])
        } else {
            json!([])
        };
        (
            200,
            json!({"items":items,"page":1,"total_pages":1}).to_string(),
        )
    };
    let reason = if status == 200 {
        "OK"
    } else if status == 401 {
        "Unauthorized"
    } else {
        "Not Found"
    };
    let response = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    drop(stream.write_all(response.as_bytes()).await);
}

pub(crate) async fn start(harness: &Harness, session: &str, base: &str, token: &str) -> StatusCode {
    let body = format!(
        "url={}&token={token}",
        base.replace(':', "%3A").replace('/', "%2F")
    );
    harness
        .post_form("/admin/mealie/start", Some(session), &body)
        .await
        .status()
}

pub(crate) async fn page_until(harness: &Harness, session: &str, expected: &str) -> String {
    for _ in 0..200 {
        let body = body_text(harness.get("/admin/mealie", Some(session)).await).await;
        if body.contains(expected) {
            return body;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let body = body_text(harness.get("/admin/mealie", Some(session)).await).await;
    assert!(body.contains(expected), "{body}");
    body
}

#[tokio::test]
async fn migration_imports_reports_failure_and_skips_imported_recipes_on_the_next_run() {
    let harness = Harness::start("mealie-migration").await;
    let session = sign_in_as(&harness, "mealie-admin", &["crabbage-admin"]).await;
    let base = mealie_server(false).await;
    assert_eq!(
        start(&harness, &session, &base, "fixture-token").await,
        StatusCode::SEE_OTHER
    );
    let summary = page_until(&harness, &session, "Failed: 1").await;
    assert!(summary.contains("Imported: 1"), "{summary}");
    assert!(summary.contains("Skipped: 0"), "{summary}");
    assert!(summary.contains("Failed: 1"), "{summary}");
    assert!(summary.contains("Broken Tart"), "{summary}");
    let ids = imported_mealie_ids(&harness.pool).await.unwrap();
    assert_eq!(ids.len(), 1);
    let owner: i64 = sqlx::query_scalar("SELECT id FROM users WHERE subject = 'mealie-admin'")
        .fetch_one(&harness.pool)
        .await
        .unwrap();
    let recipe: (i64, String) =
        sqlx::query_as("SELECT owner_id, title FROM recipes WHERE mealie_id = 'recipe-42'")
            .fetch_one(&harness.pool)
            .await
            .unwrap();
    assert_eq!(recipe, (owner, "Citrus Cakes".to_owned()));
    let rating: i64 = sqlx::query_scalar(
        "SELECT stars FROM ratings WHERE recipe_id = (SELECT id FROM recipes WHERE mealie_id = 'recipe-42') AND user_id = ?",
    )
    .bind(owner)
    .fetch_one(&harness.pool)
    .await
    .unwrap();
    assert_eq!(rating, 5);
    let categories: Vec<String> = sqlx::query_scalar(
        "SELECT categories.name FROM categories JOIN recipe_categories ON categories.id = recipe_categories.category_id JOIN recipes ON recipes.id = recipe_categories.recipe_id WHERE recipes.mealie_id = 'recipe-42' ORDER BY categories.name",
    )
    .fetch_all(&harness.pool)
    .await
    .unwrap();
    assert_eq!(categories, ["Baking", "Citrus"]);
    assert_ingredients(&harness).await.unwrap();
    let second = start(&harness, &session, &base, "fixture-token").await;
    assert_eq!(second, StatusCode::SEE_OTHER);
    let summary = page_until(&harness, &session, "Skipped: 1").await;
    assert!(summary.contains("Imported: 0"), "{summary}");
    assert_eq!(imported_mealie_ids(&harness.pool).await.unwrap().len(), 1);
}

#[tokio::test]
async fn a_bad_mealie_token_is_reported_in_the_status() {
    let harness = Harness::start("mealie-unauthorized").await;
    let session = sign_in_as(&harness, "mealie-admin", &["crabbage-admin"]).await;
    let base = mealie_server(true).await;
    assert_eq!(
        start(&harness, &session, &base, "never-return-this-token").await,
        StatusCode::SEE_OTHER
    );
    let body = page_until(&harness, &session, "Mealie rejected this API token.").await;
    assert!(!body.contains("never-return-this-token"), "{body}");
    assert_eq!(imported_mealie_ids(&harness.pool).await.unwrap().len(), 0);
}

async fn assert_ingredients(harness: &Harness) -> Result<(), Box<dyn std::error::Error>> {
    let flour = crabbage_store::catalog::ingredients::list(&harness.pool)
        .await?
        .into_iter()
        .find(|ingredient| ingredient.name == "Flour")
        .ok_or("no Flour")?;
    assert!(flour.pending_review);
    assert_eq!(flour.aliases, ["Plain Flour"]);
    let ingredients: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ingredients")
        .fetch_one(&harness.pool)
        .await?;
    assert_eq!(ingredients, 1);
    let text_line: Option<String> =
        sqlx::query_scalar("SELECT note FROM recipe_lines WHERE ingredient_id IS NULL")
            .fetch_optional(&harness.pool)
            .await?;
    assert_eq!(text_line.as_deref(), Some("salt to taste"));
    Ok(())
}
