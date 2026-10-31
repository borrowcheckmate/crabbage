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
const DRAFT: &str = r#"{"title":"Grandma's soup","servings":4,"lines":[{"amount":200,"unit":"g","name":"leeks","note":"sliced","ingredient_id":null}],"steps":[{"text":"Simmer the leeks.","lines":[0]}],"photo":{"page":0,"left":0.2,"top":0.2,"width":0.6,"height":0.6}}"#;
const NULL_PHOTO_DRAFT: &str = r#"{"title":"Grandma's soup","servings":4,"lines":[{"amount":200,"unit":"g","name":"leeks","note":"sliced","ingredient_id":null}],"steps":[{"text":"Simmer the leeks.","lines":[0]}],"photo":null}"#;

/// A loopback AI endpoint that records each request body and answers with `status` and the draft.
async fn serve_ai(status: StatusCode, draft: &'static str) -> (String, UnboundedReceiver<Value>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let base = format!("http://{}/v1", listener.local_addr().expect("address"));
    let (sender, receiver) = unbounded_channel();
    let handler = move |State(sender): State<UnboundedSender<Value>>, body: String| async move {
        drop(sender.send(serde_json::from_str(&body).unwrap_or_default()));
        let answer = json!({"choices": [{"message": {"content": draft}}]});
        (status, answer.to_string())
    };
    let router = Router::new()
        .route("/v1/chat/completions", post(handler))
        .with_state(sender);
    tokio::spawn(async move { drop(axum::serve(listener, router).await) });
    (base, receiver)
}

pub(crate) async fn with_ai(name: &str, status: StatusCode) -> (Harness, UnboundedReceiver<Value>) {
    with_ai_draft(name, status, DRAFT).await
}

async fn with_ai_draft(
    name: &str,
    status: StatusCode,
    draft: &'static str,
) -> (Harness, UnboundedReceiver<Value>) {
    let (base, requests) = serve_ai(status, draft).await;
    let harness = Harness::start(name).await;
    crabbage_store::settings::save(
        &harness.pool,
        &crabbage_store::settings::AiSettings {
            ai_enabled: true,
            provider: "custom".to_owned(),
            base_url: base,
            model: "small".to_owned(),
            api_key: None,
        },
    )
    .await
    .expect("AI settings save");
    (harness, requests)
}

pub(crate) fn png() -> Vec<u8> {
    let mut bytes = std::io::Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(100, 100)
        .write_to(&mut bytes, image::ImageFormat::Png)
        .expect("PNG fixture");
    bytes.into_inner()
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

pub(crate) async fn upload(
    harness: &Harness,
    token: Option<&str>,
    parts: &[(&str, &[u8])],
) -> Response<Body> {
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

pub(crate) fn location(response: &Response<Body>) -> String {
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    header(response, LOCATION).expect("a location")
}

async fn importer(harness: &Harness) -> i64 {
    sqlx::query_scalar("SELECT id FROM users WHERE subject = ?")
        .bind(MEMBER)
        .fetch_one(&harness.pool)
        .await
        .expect("the user exists")
}

#[tokio::test]
async fn the_photo_form_is_offered_only_when_the_ai_is_configured() {
    let (harness, _requests) = with_ai("photo-import-form", StatusCode::OK).await;
    let token = sign_in_as(&harness, MEMBER, &[]).await;
    let body = page(&harness, &token, "/recipes/import").await;
    assert!(body.contains("action=\"/recipes/import/photos\""), "{body}");
    assert!(body.contains("enctype=\"multipart/form-data\""), "{body}");
    assert!(body.contains("name=\"photo\""), "{body}");
    assert!(
        body.contains("accept=\"image/jpeg,image/png,image/webp\""),
        "{body}"
    );
    assert!(body.contains("multiple"), "{body}");
    assert!(body.contains("capture=\"environment\""), "{body}");
    assert!(body.contains("Take photo"), "{body}");
    assert!(body.contains("Choose photos"), "{body}");

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
    let bytes = png();
    let parts = [("photo", bytes.as_slice()), ("photo", bytes.as_slice())];
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
    assert!(body.contains("Source · 2 photo(s)"), "{body}");
    assert!(body.contains("data:image/jpeg;base64,"), "{body}");
    assert!(body.contains("name=\"keep_photo\""), "{body}");
    assert!(body.contains("checked"), "{body}");
}

#[tokio::test]
async fn saving_a_photo_import_attaches_the_crop_only_when_kept() {
    let (harness, _) = with_ai("photo-import-save", StatusCode::OK).await;
    let token = sign_in_as(&harness, MEMBER, &[]).await;
    let bytes = png();
    let target = location(&upload(&harness, Some(&token), &[("photo", &bytes)]).await);
    let photo_token = target
        .strip_prefix("/recipes/import?photos=")
        .expect("token");
    let recipe_draft = r#"{"title":"Soup","servings":"4","lines":[{"amount":"","unit":"","ingredient":"Leeks","note":""}],"steps":[{"text":"Simmer.","minutes":"","line_positions":[]}],"categories":[]}"#;
    for (keep_photo, expected) in [("true", true), ("", false)] {
        let mut pairs = vec![("draft", recipe_draft), ("photo_token", photo_token)];
        if !keep_photo.is_empty() {
            pairs.push(("keep_photo", keep_photo));
        }
        let body = url::form_urlencoded::Serializer::new(String::new())
            .extend_pairs(pairs)
            .finish();
        let response = harness
            .post_form("/api/save_import", Some(&token), &body)
            .await;
        assert_eq!(response.status(), StatusCode::OK);
        let id: i64 = body_text(response).await.trim().parse().expect("recipe id");
        let recipe = crabbage_store::recipes::get(&harness.pool, id, importer(&harness).await)
            .await
            .expect("read")
            .expect("recipe");
        assert_eq!(recipe.image.is_some(), expected);
    }
}

#[tokio::test]
async fn a_null_ai_photo_import_saves_without_an_image() {
    let (harness, _) = with_ai_draft("photo-import-null", StatusCode::OK, NULL_PHOTO_DRAFT).await;
    let token = sign_in_as(&harness, MEMBER, &[]).await;
    let bytes = png();
    let target = location(&upload(&harness, Some(&token), &[("photo", &bytes)]).await);
    let photo_token = target
        .strip_prefix("/recipes/import?photos=")
        .expect("token");
    let body = r#"{"title":"Soup","servings":"4","lines":[{"amount":"","unit":"","ingredient":"Leeks","note":""}],"steps":[{"text":"Simmer.","minutes":"","line_positions":[]}],"categories":[]}"#;
    let form = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("draft", body)
        .append_pair("photo_token", photo_token)
        .append_pair("keep_photo", "true")
        .finish();
    let response = harness
        .post_form("/api/save_import", Some(&token), &form)
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    let id: i64 = body_text(response).await.trim().parse().expect("recipe id");
    let recipe = crabbage_store::recipes::get(&harness.pool, id, importer(&harness).await)
        .await
        .expect("read")
        .expect("recipe");
    assert_eq!(recipe.image, None);
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
