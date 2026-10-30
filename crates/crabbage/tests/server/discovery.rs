//! The provider is discovered on first use, so the app serves while it is unreachable.

use axum::Router;
use axum::body::Body;
use axum::http::{Request, Response, StatusCode, header::LOCATION};
use leptos::config::LeptosOptions;
use tower::ServiceExt;

use crate::common::{FakeIssuer, TempDir, config, header};

async fn get(router: &Router, path: &str) -> Response<Body> {
    router
        .clone()
        .oneshot(
            Request::builder()
                .uri(path)
                .body(Body::empty())
                .expect("request builds"),
        )
        .await
        .expect("router is infallible")
}

fn free_loopback_address() -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("port is free");
    listener.local_addr().expect("address is known").to_string()
}

#[tokio::test]
async fn sign_in_is_unavailable_until_the_provider_is_reachable() {
    let address = free_loopback_address();
    let dir = TempDir::new("server-lazy-discovery");
    let pool = crabbage_store::open(dir.path())
        .await
        .expect("database opens");
    let config = config(&format!("http://{address}/"), None);
    let options = LeptosOptions::builder().output_name("crabbage").build();
    let router = crabbage::router(options, &config, pool).expect("router builds");

    assert_eq!(get(&router, "/healthz").await.status(), StatusCode::OK);
    assert_eq!(
        get(&router, "/auth/login").await.status(),
        StatusCode::SERVICE_UNAVAILABLE
    );

    let _issuer = FakeIssuer::start_on(&address).await;
    let response = get(&router, "/auth/login").await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert!(header(&response, LOCATION).is_some_and(|location| location.contains(&address)));
}
