use std::path::{Path, PathBuf};

use axum::Router;
use axum::body::Body;
use axum::http::header::{LOCATION, SET_COOKIE};
use axum::http::{Request, Response};
use crabbage::Config;
use http_body_util::BodyExt;
use leptos::config::LeptosOptions;
use sqlx::sqlite::SqlitePool;
use tower::ServiceExt;
use url::Url;

use super::FakeIssuer;

/// The cookie that carries the session token.
pub(crate) const SESSION_COOKIE: &str = "crabbage_session";

/// The cookie that binds a login to the browser that started it.
pub(crate) const LOGIN_COOKIE: &str = "crabbage_login";

/// A directory under the system temp dir that is removed when dropped.
pub(crate) struct TempDir(PathBuf);

impl TempDir {
    pub(crate) fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("crabbage-app-{}-{name}", std::process::id()));
        drop(std::fs::remove_dir_all(&path));
        Self(path)
    }

    pub(crate) fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        drop(std::fs::remove_dir_all(&self.0));
    }
}

/// The app under test: a router, the database behind it and the fake provider it signs in with.
pub(crate) struct Harness {
    pub(crate) router: Router,
    pub(crate) pool: SqlitePool,
    pub(crate) issuer: FakeIssuer,
    _dir: TempDir,
}

impl Harness {
    pub(crate) async fn start(name: &str) -> Self {
        let dir = TempDir::new(name);
        let pool = crabbage_store::open(dir.path())
            .await
            .expect("database opens");
        let issuer = FakeIssuer::start().await;
        let config = Config::from_lookup(|key| match key {
            "CRABBAGE_PUBLIC_URL" => Some("https://recipes.example.com".to_owned()),
            "OIDC_ISSUER_URL" => Some(issuer.url().to_owned()),
            "OIDC_CLIENT_ID" => Some("crabbage".to_owned()),
            "OIDC_CLIENT_SECRET" => Some("secret".to_owned()),
            _ => None,
        })
        .expect("config is valid");
        let options = LeptosOptions::builder().output_name("crabbage").build();
        let router = crabbage::router(options, &config, pool.clone())
            .await
            .expect("router builds");
        Self {
            router,
            pool,
            issuer,
            _dir: dir,
        }
    }

    pub(crate) async fn session_count(&self) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM sessions")
            .fetch_one(&self.pool)
            .await
            .expect("count reads")
    }

    /// Sends a GET with the given `Cookie` header value.
    pub(crate) async fn get_with_cookies(&self, path: &str, cookies: &str) -> Response<Body> {
        let request = Request::builder()
            .uri(path)
            .header("cookie", cookies)
            .body(Body::empty())
            .expect("request builds");
        self.send(request).await
    }

    pub(crate) async fn send(&self, request: Request<Body>) -> Response<Body> {
        self.router
            .clone()
            .oneshot(request)
            .await
            .expect("router is infallible")
    }

    /// Sends a GET, with the session cookie when `cookie` is given.
    pub(crate) async fn get(&self, path: &str, cookie: Option<&str>) -> Response<Body> {
        let mut builder = Request::builder().uri(path);
        if let Some(token) = cookie {
            builder = builder.header("cookie", format!("{SESSION_COOKIE}={token}"));
        }
        self.send(builder.body(Body::empty()).expect("request builds"))
            .await
    }

    /// Runs the whole sign-in flow, as the browser that started it, and returns the callback
    /// response, which sets the session.
    pub(crate) async fn sign_in(&self) -> Response<Body> {
        let login = self.get("/auth/login", None).await;
        let (state, nonce) = authorization_params(&login);
        let login_cookie = cookie_pair(&login, LOGIN_COOKIE).expect("login sets the login cookie");
        self.issuer.expect_nonce(&nonce);
        self.get_with_cookies(
            &format!("/auth/callback?code=code&state={state}"),
            &login_cookie,
        )
        .await
    }
}

/// The `state` and `nonce` sent in the redirect to the provider.
pub(crate) fn authorization_params(response: &Response<Body>) -> (String, String) {
    let location = header(response, LOCATION).expect("redirect has a location");
    let url = Url::parse(&location).expect("location is a URL");
    let param = |name: &str| {
        url.query_pairs()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.into_owned())
            .expect("authorization request has the parameter")
    };
    (param("state"), param("nonce"))
}

/// The full `Set-Cookie` header for the cookie `name`, if the response sets one.
pub(crate) fn set_cookie(response: &Response<Body>, name: &str) -> Option<String> {
    response
        .headers()
        .get_all(SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .find(|value| value.starts_with(&format!("{name}=")))
        .map(str::to_owned)
}

/// The `name=value` pair the response sets for the cookie `name`, if any.
pub(crate) fn cookie_pair(response: &Response<Body>, name: &str) -> Option<String> {
    let cookie = set_cookie(response, name)?;
    cookie.split(';').next().map(str::to_owned)
}

/// The value the response sets for the cookie `name`, if any and not empty.
pub(crate) fn cookie_value(response: &Response<Body>, name: &str) -> Option<String> {
    cookie_pair(response, name)?
        .strip_prefix(&format!("{name}="))
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

/// The session token the response sets, if any.
pub(crate) fn session_token(response: &Response<Body>) -> Option<String> {
    cookie_value(response, SESSION_COOKIE)
}

/// The value of a response header, if present and valid text.
pub(crate) fn header(response: &Response<Body>, name: axum::http::HeaderName) -> Option<String> {
    response
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
}

/// Reads the response body as text.
pub(crate) async fn body_text(response: Response<Body>) -> String {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body reads")
        .to_bytes();
    String::from_utf8_lossy(&bytes).into_owned()
}
