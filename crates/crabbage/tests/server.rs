//! Checks the HTTP routes: the health endpoint and the server-rendered shell.

use std::error::Error;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use leptos::config::LeptosOptions;
use tower::ServiceExt;

async fn get_page(path: &str) -> Result<(StatusCode, String), Box<dyn Error>> {
    let options = LeptosOptions::builder().output_name("crabbage").build();
    let request = Request::builder().uri(path).body(Body::empty())?;
    let response = crabbage::router(options).oneshot(request).await?;
    let status = response.status();
    let bytes = response.into_body().collect().await?.to_bytes();
    Ok((status, String::from_utf8_lossy(&bytes).into_owned()))
}

#[tokio::test]
async fn healthz_is_ok() {
    let (status, body) = get_page("/healthz").await.expect("request succeeds");
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "ok");
}

#[tokio::test]
async fn home_renders_the_shell_with_the_title() {
    let (status, body) = get_page("/").await.expect("request succeeds");
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("<title>crabbage</title>"), "{body}");
}
