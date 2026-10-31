//! The links from steps to ingredient lines through the recipe endpoints: a save keeps the links the
//! form sent, a removed line takes its links with it, and a link to a blank line follows the line
//! that is kept.

use axum::http::StatusCode;
use serde_json::{Value, json};

use crate::common::{Harness, body_text, sign_in_as};
use crate::recipes::{call, create, view};

const OWNER: &str = "owner";

/// The form draft a recipe page would send for the record `recipe`, with its links as they are.
fn draft_of(recipe: &Value) -> Value {
    let lines: Vec<Value> = recipe["lines"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|line| {
            json!({
                "amount": line["amount"].as_f64().map(|amount| amount.to_string()).unwrap_or_default(),
                "unit": line["unit"].as_str().unwrap_or_default(),
                "ingredient": line["ingredient"].as_str().unwrap_or_default(),
                "note": line["note"].as_str().unwrap_or_default(),
            })
        })
        .collect();
    let steps: Vec<Value> = recipe["steps"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|step| {
            json!({
                "text": step["text"],
                "minutes": step["duration_seconds"].as_i64().map(|seconds| (seconds / 60).to_string()).unwrap_or_default(),
                "line_positions": step["line_positions"],
            })
        })
        .collect();
    json!({
        "title": recipe["title"],
        "servings": recipe["servings"].to_string(),
        "lines": lines,
        "steps": steps,
    })
}

/// Replaces recipe `id` with `draft` as `token`, and asserts the save succeeded.
async fn save(harness: &Harness, token: &str, id: i64, draft: &Value) {
    let (status, body) = call(
        harness,
        token,
        "update_recipe",
        &[("id", &id.to_string()), ("draft", &draft.to_string())],
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

/// The step positions of recipe `recipe`, in step order.
fn links(recipe: &Value) -> Value {
    Value::Array(
        recipe["steps"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|step| step["line_positions"].clone())
            .collect(),
    )
}

/// The ingredient names of the lines step `index` links to.
fn linked_names(recipe: &Value, index: usize) -> Vec<String> {
    let lines = recipe["lines"].as_array().cloned().unwrap_or_default();
    let steps = recipe["steps"].as_array().cloned().unwrap_or_default();
    let positions = steps
        .get(index)
        .map(|step| step["line_positions"].clone())
        .unwrap_or_default();
    positions
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_u64)
        .filter_map(|position| {
            let line = lines.get(usize::try_from(position).ok()?)?;
            line["ingredient"].as_str().map(str::to_owned)
        })
        .collect()
}

#[tokio::test]
async fn an_update_that_does_not_touch_the_links_keeps_them() {
    let harness = Harness::start("step-lines-keep").await;
    let token = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &token, "Pancakes").await;
    let before = view(&harness, &token, id).await;
    assert_eq!(links(&before), json!([[1], [0, 2], []]));

    save(&harness, &token, id, &draft_of(&before)).await;

    let after = view(&harness, &token, id).await;
    assert_eq!(links(&after), json!([[1], [0, 2], []]));
    assert_eq!(linked_names(&after, 1), ["Flour", "Salt"]);
}

#[tokio::test]
async fn removing_a_line_drops_its_links_and_shifts_the_later_ones() {
    let harness = Harness::start("step-lines-remove").await;
    let token = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &token, "Pancakes").await;
    let before = view(&harness, &token, id).await;

    let mut draft = draft_of(&before);
    draft["lines"]
        .as_array_mut()
        .expect("lines are a list")
        .remove(1);
    draft["steps"][0]["line_positions"] = json!([]);
    draft["steps"][1]["line_positions"] = json!([0, 1]);
    save(&harness, &token, id, &draft).await;

    let after = view(&harness, &token, id).await;
    assert_eq!(after["lines"].as_array().map(Vec::len), Some(2));
    assert_eq!(links(&after), json!([[], [0, 1], []]));
    assert_eq!(linked_names(&after, 1), ["Flour", "Salt"]);
}

#[tokio::test]
async fn a_link_follows_its_line_past_a_blank_row_and_a_missing_link_is_dropped() {
    let harness = Harness::start("step-lines-blank").await;
    let token = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &token, "Pancakes").await;
    let before = view(&harness, &token, id).await;

    let mut draft = draft_of(&before);
    let blank = json!({"amount": "", "unit": "", "ingredient": "", "note": ""});
    draft["lines"] = json!([blank, draft["lines"][0], draft["lines"][1]]);
    draft["steps"][0]["line_positions"] = json!([2]);
    draft["steps"][1]["line_positions"] = json!([1, 9]);
    save(&harness, &token, id, &draft).await;

    let after = view(&harness, &token, id).await;
    assert_eq!(after["lines"].as_array().map(Vec::len), Some(2));
    assert_eq!(links(&after), json!([[1], [0], []]));
    assert_eq!(linked_names(&after, 0), ["Egg"]);
    assert_eq!(linked_names(&after, 1), ["Flour"]);
}

#[tokio::test]
async fn the_read_model_gives_each_step_the_positions_of_its_lines() {
    let harness = Harness::start("step-lines-read").await;
    let token = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &token, "Pancakes").await;
    let recipe = view(&harness, &token, id).await;
    assert_eq!(recipe["steps"][0]["line_positions"], json!([1]));
    assert_eq!(recipe["steps"][1]["line_positions"], json!([0, 2]));
    assert_eq!(recipe["steps"][2]["line_positions"], json!([]));
    let body = body_text(
        harness
            .get(&format!("/recipes/{id}/edit"), Some(&token))
            .await,
    )
    .await;
    assert!(body.contains("Ingredients for this step"), "{body}");
}
