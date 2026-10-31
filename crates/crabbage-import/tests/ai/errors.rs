use std::time::Duration;

use crabbage_import::ai::AiError;

use crate::{KEY, ai, completion, failure_of, prompt, refused, silent};

#[test]
fn every_error_has_a_clear_message() {
    let messages = [
        (AiError::Timeout, "the AI service did not answer in time"),
        (AiError::Status(401), "the AI service rejected the API key"),
        (AiError::Status(403), "the AI service rejected the API key"),
        (
            AiError::Status(429),
            "the AI service is busy, try again later",
        ),
        (
            AiError::Status(500),
            "the AI service answered with status 500",
        ),
        (
            AiError::Network("connection reset".to_owned()),
            "the AI service could not be reached: connection reset",
        ),
        (
            AiError::Invalid("the title is blank".to_owned()),
            "the AI service gave an unusable draft: the title is blank",
        ),
    ];
    for (error, text) in messages {
        assert_eq!(error.to_string(), text);
    }
}

#[tokio::test]
async fn a_server_that_never_answers_times_out_with_the_key_set() {
    let base = silent().await;
    let error = ai(&format!("{base}/v1"), Some(KEY))
        .with_timeout(Duration::from_millis(200))
        .complete(&prompt("Lemon pancakes"))
        .await;
    assert_eq!(error, Err(AiError::Timeout));
}

#[tokio::test]
async fn a_rejected_key_is_reported() {
    assert_eq!(
        failure_of("401 Unauthorized", "").await,
        AiError::Status(401)
    );
}

#[tokio::test]
async fn a_busy_service_and_a_server_error_are_reported() {
    assert_eq!(
        failure_of("429 Too Many Requests", "").await,
        AiError::Status(429)
    );
    assert_eq!(
        failure_of("500 Internal Server Error", "").await,
        AiError::Status(500)
    );
}

#[tokio::test]
async fn an_unreachable_service_is_a_network_error() {
    let base = refused().await;
    let error = ai(&format!("{base}/v1"), Some(KEY))
        .complete(&prompt("Lemon pancakes"))
        .await
        .expect_err("an error");
    assert!(matches!(error, AiError::Network(_)), "{error:?}");
}

#[tokio::test]
async fn an_answer_without_a_message_is_invalid() {
    assert_eq!(
        failure_of("200 OK", "{}").await,
        AiError::Invalid("the response has no message".to_owned())
    );
    assert_eq!(
        failure_of("200 OK", "not json").await,
        AiError::Invalid("the response has no message".to_owned())
    );
}

#[tokio::test]
async fn an_answer_with_a_bad_draft_is_invalid() {
    let error = failure_of("200 OK", &completion("not json")).await;
    assert!(
        matches!(&error, AiError::Invalid(text) if text.contains("not the expected JSON")),
        "{error:?}"
    );
}

#[tokio::test]
async fn the_key_never_appears_in_errors_or_the_client() {
    let timeout_base = silent().await;
    let timed_out = ai(&format!("{timeout_base}/v1"), Some(KEY))
        .with_timeout(Duration::from_millis(200))
        .complete(&prompt("Lemon pancakes"))
        .await
        .expect_err("an error");
    let refused_base = refused().await;
    let unreachable = ai(&format!("{refused_base}/v1"), Some(KEY))
        .complete(&prompt("Lemon pancakes"))
        .await
        .expect_err("an error");
    let errors = [
        timed_out,
        failure_of("401 Unauthorized", "").await,
        failure_of("500 Internal Server Error", "").await,
        unreachable,
        failure_of("200 OK", &completion("not json")).await,
    ];
    for error in errors {
        assert!(!format!("{error}").contains(KEY), "{error}");
        assert!(!format!("{error:?}").contains(KEY), "{error:?}");
    }
    let client = ai("http://127.0.0.1:9/v1", Some(KEY));
    let debug = format!("{client:?}");
    assert!(!debug.contains(KEY), "{debug}");
    assert!(debug.contains("<redacted>"), "{debug}");
}
