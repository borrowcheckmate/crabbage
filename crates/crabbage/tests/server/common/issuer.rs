//! A fake `OpenID` Connect provider on the loopback interface.

use std::sync::{Arc, Mutex, OnceLock};

use axum::Router;
use axum::http::{StatusCode, header::CONTENT_TYPE};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use openidconnect::core::CoreRsaPrivateSigningKey;
use openidconnect::core::{CoreJsonWebKey, CoreJwsSigningAlgorithm};
use openidconnect::{JsonWebKeyId, PrivateSigningKey};
use rsa::RsaPrivateKey;
use rsa::pkcs1::EncodeRsaPrivateKey;
use rsa::pkcs8::LineEnding;
use rsa::rand_core::OsRng;
use serde_json::json;
use tokio::net::TcpListener;

const KEY_ID: &str = "test-key";
const CLIENT_ID: &str = "crabbage";

/// The RSA key used by every fake issuer in this test binary, generated on first use.
fn signing_key() -> &'static CoreRsaPrivateSigningKey {
    static KEY: OnceLock<CoreRsaPrivateSigningKey> = OnceLock::new();
    KEY.get_or_init(|| {
        let private = RsaPrivateKey::new(&mut OsRng, 2048).expect("key generates");
        let pem = private
            .to_pkcs1_pem(LineEnding::LF)
            .expect("key encodes")
            .to_string();
        CoreRsaPrivateSigningKey::from_pem(&pem, Some(JsonWebKeyId::new(KEY_ID.to_owned())))
            .expect("key loads")
    })
}

#[derive(Default)]
struct Expectations {
    nonce: Option<String>,
    groups: Vec<String>,
}

/// A running fake provider. Its ID tokens carry the nonce set with [`FakeIssuer::expect_nonce`].
#[derive(Clone)]
pub(crate) struct FakeIssuer {
    url: String,
    expectations: Arc<Mutex<Expectations>>,
}

impl FakeIssuer {
    /// Starts the provider on a free loopback port.
    pub(crate) async fn start() -> Self {
        Self::start_on("127.0.0.1:0").await
    }

    /// Starts the provider on `address`, a loopback socket address.
    /// The key is generated before serving so no request waits on it past the client timeout.
    pub(crate) async fn start_on(address: &str) -> Self {
        tokio::task::spawn_blocking(signing_key)
            .await
            .expect("key generates");
        let listener = TcpListener::bind(address).await.expect("address is free");
        let address = listener.local_addr().expect("address is known");
        let issuer = Self {
            url: format!("http://{address}/"),
            expectations: Arc::new(Mutex::new(Expectations::default())),
        };
        let router = Router::new()
            .route("/.well-known/openid-configuration", get(discovery))
            .route("/jwks", get(jwks))
            .route("/token", post(token))
            .with_state(issuer.clone());
        tokio::spawn(async move {
            drop(axum::serve(listener, router).await);
        });
        issuer
    }

    /// The issuer URL, with a trailing slash. Its endpoints are relative to it.
    pub(crate) fn url(&self) -> &str {
        &self.url
    }

    /// Makes the next ID tokens carry `nonce`.
    pub(crate) fn expect_nonce(&self, nonce: &str) {
        self.expectations.lock().expect("not poisoned").nonce = Some(nonce.to_owned());
    }

    /// Makes the next ID tokens carry the `groups` claim `groups`.
    pub(crate) fn set_groups(&self, groups: &[&str]) {
        self.expectations.lock().expect("not poisoned").groups =
            groups.iter().map(|group| (*group).to_owned()).collect();
    }

    fn id_token(&self) -> String {
        let (nonce, groups) = {
            let expectations = self.expectations.lock().expect("not poisoned");
            (expectations.nonce.clone(), expectations.groups.clone())
        };
        let now = 1_900_000_000_u64;
        let header = json!({ "alg": "RS256", "kid": KEY_ID, "typ": "JWT" });
        let payload = json!({
            "iss": self.url,
            "sub": "subject-1",
            "aud": CLIENT_ID,
            "exp": now + 3600,
            "iat": now,
            "nonce": nonce,
            "name": "Ada Lovelace",
            "preferred_username": "ada",
            "groups": groups,
        });
        let signing_input = format!(
            "{}.{}",
            URL_SAFE_NO_PAD.encode(header.to_string()),
            URL_SAFE_NO_PAD.encode(payload.to_string())
        );
        let signature = signing_key()
            .sign(
                &CoreJwsSigningAlgorithm::RsaSsaPkcs1V15Sha256,
                signing_input.as_bytes(),
            )
            .expect("token signs");
        format!("{signing_input}.{}", URL_SAFE_NO_PAD.encode(signature))
    }
}

async fn discovery(axum::extract::State(issuer): axum::extract::State<FakeIssuer>) -> Response {
    let document = json!({
        "issuer": issuer.url,
        "authorization_endpoint": format!("{}authorize", issuer.url),
        "token_endpoint": format!("{}token", issuer.url),
        "jwks_uri": format!("{}jwks", issuer.url),
        "response_types_supported": ["code"],
        "subject_types_supported": ["public"],
        "id_token_signing_alg_values_supported": ["RS256"],
    });
    json_response(&document.to_string())
}

async fn jwks() -> Response {
    let key: CoreJsonWebKey = signing_key().as_verification_key();
    json_response(&json!({ "keys": [key] }).to_string())
}

async fn token(
    axum::extract::State(issuer): axum::extract::State<FakeIssuer>,
    body: String,
) -> Response {
    if !body.contains("code_verifier=") {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let document = json!({
        "access_token": "access-token",
        "token_type": "Bearer",
        "expires_in": 3600,
        "id_token": issuer.id_token(),
    });
    json_response(&document.to_string())
}

fn json_response(body: &str) -> Response {
    ([(CONTENT_TYPE, "application/json")], body.to_owned()).into_response()
}
