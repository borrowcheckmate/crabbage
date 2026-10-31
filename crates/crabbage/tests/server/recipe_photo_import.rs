//! Importing a recipe from photos: the form is offered only when the AI is configured, uploads go to
//! the AI as data URLs, and the draft it answers with is shown in the import form for the same user.

use axum::Router;
use axum::body::Body;
use axum::extract::State;
use axum::http::header::LOCATION;
use axum::http::{Request, Response, StatusCode};
use axum::routing::post;
use serde_json::{Value, json};
use tokio::net::TcpListener;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use crate::common::{Harness, SESSION_COOKIE, body_text, header, sign_in_as};

const BOUNDARY: &str = "crabbage-test-boundary";
const MEMBER: &str = "photographer";
const PNG: &[u8] = b"\x89PNG\r\n\x1a\npng";
const DRAFT: &str = r#"{"title":"Grandma's soup","servings":4,"lines":[{"amount":200,"unit":"g","name":"leeks","note":"sliced","ingredient_id":null}],"steps":[{"text":"Simmer the leeks.","lines":[0]}]}"#;

/// A loopback AI endpoint that records each request body and answers with `status` and the draft.
async fn serve_ai(status: StatusCode) -> (String, UnboundedReceiver<Value>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let base = format!("http://{}/v1", listener.local_addr().expect("address"));
    let (sender, receiver) = unbounded_channel();
    let handler = move |State(sender): State<UnboundedSender<Value>>, body: String| async move {
        drop(sender.send(serde_json::from_str(&body).unwrap_or_default()));
        let answer = json!({"choices": [{"message": {"content": DRAFT}}]});
        (status, answer.to_string())
    };
    let router = Router::new()
        .route("/v1/chat/completions", post(handler))
        .with_state(sender);
    tokio::spawn(async move { drop(axum::serve(listener, router).await) });
    (base, receiver)
}

pub(crate) async fn with_ai(name: &str, status: StatusCode) -> (Harness, UnboundedReceiver<Value>) {
    let (base, requests) = serve_ai(status).await;
    let extra = [("AI_BASE_URL", base.as_str()), ("AI_MODEL", "small")];
    (Harness::start_with(name, None, &extra).await, requests)
}

fn multipart(parts: &[(&str, &[u8])]) -> Vec<u8> {
    let mut body = Vec::new();
    for (name, bytes) in parts {
        body.extend_from_slice(
            format!(
                "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"{name}\"; filename=\"p\"\r\n\r\n"
            )
            .as_bytes(),
        );
        body.extend_from_slice(bytes);
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(format!("--{BOUNDARY}--\r\n").as_bytes());
    body
}

async fn upload(harness: &Harness, token: Option<&str>, parts: &[(&str, &[u8])]) -> Response<Body> {
    let mut builder = Request::builder()
        .method("POST")
        .uri("/recipes/import/photos")
        .header(
            "content-type",
            format!("multipart/form-data; boundary={BOUNDARY}"),
        );
    if let Some(token) = token {
        builder = builder.header("cookie", format!("{SESSION_COOKIE}={token}"));
    }
    let body = Body::from(multipart(parts));
    harness.send(builder.body(body).expect("request")).await
}

async fn page(harness: &Harness, token: &str, path: &str) -> String {
    let response = harness.get(path, Some(token)).await;
    assert_eq!(response.status(), StatusCode::OK, "{path}");
    body_text(response).await
}

fn location(response: &Response<Body>) -> String {
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    header(response, LOCATION).expect("a location")
}

#[tokio::test]
async fn the_photo_form_is_offered_only_when_the_ai_is_configured() {
    let (harness, _requests) = with_ai("photo-import-form", StatusCode::OK).await;
    let token = sign_in_as(&harness, MEMBER, &[]).await;
    let body = page(&harness, &token, "/recipes/import").await;
    assert!(body.contains("action=\"/recipes/import/photos\""), "{body}");
    assert!(body.contains("enctype=\"multipart/form-data\""), "{body}");
    assert!(body.contains("name=\"photo\""), "{body}");
    assert!(body.contains("accept=\"image/*\""), "{body}");
    assert!(body.contains("multiple"), "{body}");
    assert!(!body.contains("capture"), "{body}");

    let off = Harness::start("photo-import-off").await;
    let token = sign_in_as(&off, MEMBER, &[]).await;
    let body = page(&off, &token, "/recipes/import").await;
    assert!(!body.contains("/recipes/import/photos"), "{body}");
    let response = upload(&off, Some(&token), &[("photo", PNG)]).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn signed_out_uploads_are_sent_to_sign_in() {
    let (harness, mut requests) = with_ai("photo-import-signed-out", StatusCode::OK).await;
    let response = upload(&harness, None, &[("photo", PNG)]).await;
    assert_eq!(location(&response), "/auth/login");
    assert!(requests.try_recv().is_err());
}

#[tokio::test]
async fn two_photos_become_one_ai_request_and_a_draft_to_review() {
    let (harness, mut requests) = with_ai("photo-import-draft", StatusCode::OK).await;
    let token = sign_in_as(&harness, MEMBER, &[]).await;
    let parts = [("photo", PNG), ("photo", PNG)];
    let target = location(&upload(&harness, Some(&token), &parts).await);
    assert!(target.starts_with("/recipes/import?photos="), "{target}");
    assert_eq!(target.len(), "/recipes/import?photos=".len() + 32);
    let request = requests.try_recv().expect("one request");
    assert!(requests.try_recv().is_err());
    let content = request["messages"][1]["content"].as_array().expect("parts");
    let images = content
        .iter()
        .filter(|part| {
            part["image_url"]["url"]
                .as_str()
                .is_some_and(|url| url.starts_with("data:image/png;base64,"))
        })
        .count();
    assert_eq!(images, 2, "{request}");
    let body = page(&harness, &token, &target).await;
    assert!(body.contains("Grandma"), "{body}");
    assert!(body.contains("value=\"leeks\""), "{body}");
    assert!(body.contains("value=\"sliced\""), "{body}");
    assert!(body.contains("Simmer the leeks."), "{body}");
    assert!(body.contains("Read from 2 photo(s)."), "{body}");
}

#[tokio::test]
async fn files_that_are_not_images_are_refused_with_a_message() {
    let (harness, mut requests) = with_ai("photo-import-invalid", StatusCode::OK).await;
    let token = sign_in_as(&harness, MEMBER, &[]).await;
    let parts = [("photo", &b"just text"[..]), ("other", PNG)];
    let target = location(&upload(&harness, Some(&token), &parts).await);
    assert_eq!(target, "/recipes/import?photo_error=invalid");
    assert!(requests.try_recv().is_err());
    let body = page(&harness, &token, &target).await;
    assert!(body.contains("None of the files is a JPEG"), "{body}");
}

#[tokio::test]
async fn an_ai_failure_is_reported_on_the_import_page() {
    let (harness, _requests) = with_ai("photo-import-ai", StatusCode::INTERNAL_SERVER_ERROR).await;
    let token = sign_in_as(&harness, MEMBER, &[]).await;
    let target = location(&upload(&harness, Some(&token), &[("photo", PNG)]).await);
    assert_eq!(target, "/recipes/import?photo_error=ai");
    let body = page(&harness, &token, &target).await;
    assert!(body.contains("could not read a recipe"), "{body}");
}

#[tokio::test]
async fn a_draft_is_only_shown_to_its_owner_and_unknown_tokens_have_expired() {
    let (harness, _requests) = with_ai("photo-import-owner", StatusCode::OK).await;
    let owner = sign_in_as(&harness, MEMBER, &[]).await;
    let other = sign_in_as(&harness, "someone-else", &[]).await;
    let target = location(&upload(&harness, Some(&owner), &[("photo", PNG)]).await);
    let stolen = page(&harness, &other, &target).await;
    assert!(stolen.contains("photo import has expired"), "{stolen}");
    assert!(!stolen.contains("Grandma"), "{stolen}");
    let unknown = page(&harness, &owner, "/recipes/import?photos=abc").await;
    assert!(unknown.contains("photo import has expired"), "{unknown}");
}
