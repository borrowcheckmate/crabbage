use std::time::Duration;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use crabbage_import::ai::AiError;
use serde_json::json;

use crate::{VALID, ai, body_json, completion, header, prompt, request_path, serve, silent};

#[tokio::test]
async fn the_request_goes_to_chat_completions_with_or_without_a_trailing_slash() {
    let (base, mut requests) = serve("200 OK", &completion(VALID)).await;
    for suffix in ["/v1", "/v1/"] {
        ai(&format!("{base}{suffix}"), None)
            .complete(&prompt("Lemon pancakes"))
            .await
            .expect("draft");
        let request = requests.try_recv().expect("request");
        assert!(request.starts_with("POST "), "{request}");
        assert_eq!(request_path(&request), "/v1/chat/completions");
        assert_eq!(header(&request, "content-type"), Some("application/json"));
    }
}

#[tokio::test]
async fn the_key_is_sent_as_a_bearer_token_when_set() {
    let (base, mut requests) = serve("200 OK", &completion(VALID)).await;
    ai(&format!("{base}/v1"), Some("test-key"))
        .complete(&prompt("Lemon pancakes"))
        .await
        .expect("draft");
    let request = requests.try_recv().expect("request");
    assert_eq!(header(&request, "authorization"), Some("Bearer test-key"));
}

#[tokio::test]
async fn no_authorization_header_is_sent_without_a_key() {
    let (base, mut requests) = serve("200 OK", &completion(VALID)).await;
    ai(&format!("{base}/v1"), None)
        .complete(&prompt("Lemon pancakes"))
        .await
        .expect("draft");
    let request = requests.try_recv().expect("request");
    assert_eq!(header(&request, "authorization"), None);
}

#[tokio::test]
async fn the_body_has_the_model_instructions_text_and_photo() {
    let (base, mut requests) = serve("200 OK", &completion(VALID)).await;
    ai(&format!("{base}/v1"), None)
        .complete(&prompt("Lemon pancakes"))
        .await
        .expect("draft");
    let body = body_json(&requests.try_recv().expect("request"));
    assert_eq!(body["model"], "test-model");
    assert_eq!(body["response_format"]["type"], "json_object");
    assert_eq!(body["messages"][0]["role"], "system");
    assert!(
        body["messages"][0]["content"]
            .as_str()
            .is_some_and(|text| text.starts_with("Read the recipe.\n\n"))
    );
    assert_eq!(body["messages"][1]["role"], "user");
    assert_eq!(
        body["messages"][1]["content"][0],
        json!({"type": "text", "text": "Lemon pancakes"})
    );
    assert_eq!(body["messages"][1]["content"][1]["type"], "image_url");
    let url = body["messages"][1]["content"][1]["image_url"]["url"]
        .as_str()
        .expect("image url");
    let encoded = url
        .strip_prefix("data:image/png;base64,")
        .expect("png data url");
    assert_eq!(
        STANDARD.decode(encoded).expect("base64"),
        [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
    );
}

#[tokio::test]
async fn the_text_part_is_left_out_when_the_text_is_empty() {
    let (base, mut requests) = serve("200 OK", &completion(VALID)).await;
    ai(&format!("{base}/v1"), None)
        .complete(&prompt(""))
        .await
        .expect("draft");
    let body = body_json(&requests.try_recv().expect("request"));
    let parts = body["messages"][1]["content"].as_array().expect("parts");
    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0]["type"], "image_url");
}

#[tokio::test]
async fn a_valid_answer_becomes_the_draft() {
    let (base, _) = serve("200 OK", &completion(VALID)).await;
    let draft = ai(&format!("{base}/v1"), None)
        .complete(&prompt("Lemon pancakes"))
        .await
        .expect("draft");
    assert_eq!(draft.title, "Lemon Pancakes");
    assert_eq!(draft.lines.len(), 2);
    assert_eq!(draft.steps.len(), 1);
}

#[tokio::test]
async fn a_server_that_never_answers_times_out() {
    let base = silent().await;
    let client = ai(&format!("{base}/v1"), None).with_timeout(Duration::from_millis(200));
    assert_eq!(
        client.complete(&prompt("Lemon pancakes")).await,
        Err(AiError::Timeout)
    );
}
