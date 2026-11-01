use crabbage_import::ai::AiClient;
use crabbage_import::pipeline::{complete, from_draft};
use serde_json::Value;
use url::Url;

use crate::server::{VALID, completion, serve};
use crate::{candidate, draft};

#[expect(clippy::expect_used, reason = "the test client URL is always valid")]
fn client(base: &str) -> AiClient {
    AiClient::new(
        Url::parse(&format!("{base}/v1")).expect("url"),
        None,
        "test-model".to_owned(),
    )
}

#[expect(
    clippy::expect_used,
    reason = "a request without a JSON body is a failed test"
)]
fn body_of(request: &str) -> Value {
    let (_, body) = request.split_once("\r\n\r\n").expect("a body");
    serde_json::from_str(body).expect("a JSON body")
}

#[tokio::test]
async fn the_ai_is_called_for_a_complete_draft_and_prompt_has_candidates_and_categories() {
    let (base, mut requests) = serve("200 OK", &completion(VALID)).await;
    let flour = candidate(1, "flour");
    let page = draft(&["200 g flour"], &["Mix the flour."]);
    let result = complete(
        &page,
        &[flour],
        &[(7, "Breakfast")],
        Some(&client(&base)),
        "en",
    )
    .await;
    assert_eq!(result.title, "Fluffy pancakes");
    let request = requests.try_recv().expect("AI called for complete parse");
    let body = body_of(&request);
    let text = body["messages"][1]["content"][0]["text"]
        .as_str()
        .unwrap_or_default();
    assert!(text.contains("1: flour"), "{text}");
    assert!(text.contains("7: Breakfast"), "{text}");
    let instructions = body["messages"][0]["content"].as_str().unwrap_or_default();
    assert!(
        instructions.contains("only the ingredient as bought"),
        "{instructions}"
    );
    assert!(
        instructions.contains("put all such text in") && instructions.contains("\"note\""),
        "{instructions}"
    );
}

#[tokio::test]
async fn the_ai_is_called_for_an_unmatched_line_and_its_draft_is_merged() {
    let (base, mut requests) = serve("200 OK", &completion(VALID)).await;
    let flour = candidate(1, "flour");
    let page = draft(&["200 g flour", "2 eggs"], &["Whisk the flour and eggs."]);
    let result = complete(&page, &[flour], &[], Some(&client(&base)), "en").await;
    assert_eq!(result.title, "Fluffy pancakes");
    assert_eq!(result.lines[0].ingredient_id, Some(1));
    assert_eq!(result.unmatched, vec!["eggs".to_owned()]);
    assert_eq!(result.ai_error, None);
    let request = requests.try_recv().expect("one request");
    assert!(requests.try_recv().is_err());
    let body = body_of(&request);
    let text = body["messages"][1]["content"][0]["text"]
        .as_str()
        .unwrap_or_default();
    assert!(text.contains("Title: Lemon Pancakes"), "{text}");
    assert!(text.contains("1. 2 eggs"), "{text}");
    let instructions = body["messages"][0]["content"].as_str().unwrap_or_default();
    assert!(
        instructions.contains("language with the code en"),
        "{instructions}"
    );
}

#[tokio::test]
async fn the_ai_is_called_when_the_page_is_in_another_language() {
    let (base, mut requests) = serve("200 OK", &completion(VALID)).await;
    let flour = candidate(1, "flour");
    let mut page = draft(&["200 g flour"], &["Mix the flour."]);
    page.language = Some("nl-NL".to_owned());
    let result = complete(&page, &[flour], &[], Some(&client(&base)), "en").await;
    assert_eq!(result.title, "Fluffy pancakes");
    assert!(requests.try_recv().is_ok());
}

#[tokio::test]
async fn an_ai_failure_keeps_the_reading_and_sets_the_error() {
    let (base, _requests) = serve("500 Internal Server Error", "{}").await;
    let flour = candidate(1, "flour");
    let page = draft(&["200 g flour", "2 eggs"], &["Whisk the flour and eggs."]);
    let result = complete(&page, &[flour], &[], Some(&client(&base)), "en").await;
    let reading = from_draft(&page, &[flour]);
    assert_eq!(
        result.ai_error.as_deref(),
        Some("the AI service answered with status 500")
    );
    assert_eq!(
        result,
        crabbage_import::pipeline::Imported {
            ai_error: result.ai_error.clone(),
            ..reading
        }
    );
}

#[tokio::test]
async fn without_an_ai_config_the_reading_is_returned_unchanged() {
    let flour = candidate(1, "flour");
    let page = draft(&["200 g flour", "2 eggs"], &["Whisk the flour and eggs."]);
    let result = complete(&page, &[flour], &[], None, "en").await;
    assert_eq!(result, from_draft(&page, &[flour]));
}
