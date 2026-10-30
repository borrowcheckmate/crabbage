//! The `OpenID` Connect client: discovery, the authorization request and the ID token check.

use std::error::Error;
use std::time::Duration;

use crabbage_store::login_attempts::LoginAttempt;
use openidconnect::core::{
    CoreAuthDisplay, CoreAuthPrompt, CoreAuthenticationFlow, CoreErrorResponseType,
    CoreGenderClaim, CoreJsonWebKey, CoreJweContentEncryptionAlgorithm, CoreJwsSigningAlgorithm,
    CoreProviderMetadata, CoreRevocableToken, CoreRevocationErrorResponse,
    CoreTokenIntrospectionResponse, CoreTokenType,
};
use openidconnect::{
    AdditionalClaims, AuthorizationCode, Client, ClientId, ClientSecret, CsrfToken,
    EmptyExtraTokenFields, EndpointMaybeSet, EndpointNotSet, EndpointSet, IdTokenFields, IssuerUrl,
    Nonce, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, StandardErrorResponse,
    StandardTokenResponse, TokenResponse, reqwest, url::Url,
};
use serde::{Deserialize, Serialize};

use crate::config::Config;

pub(crate) type HttpClient = reqwest::Client;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Profile {
    name: Option<String>,
    preferred_username: Option<String>,
    groups: Option<Vec<String>>,
}

impl AdditionalClaims for Profile {}

type ProfileTokenResponse = StandardTokenResponse<
    IdTokenFields<
        Profile,
        EmptyExtraTokenFields,
        CoreGenderClaim,
        CoreJweContentEncryptionAlgorithm,
        CoreJwsSigningAlgorithm,
    >,
    CoreTokenType,
>;

type ProfileClient = Client<
    Profile,
    CoreAuthDisplay,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJsonWebKey,
    CoreAuthPrompt,
    StandardErrorResponse<CoreErrorResponseType>,
    ProfileTokenResponse,
    CoreTokenIntrospectionResponse,
    CoreRevocableToken,
    CoreRevocationErrorResponse,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

pub(crate) struct Identity {
    pub(crate) subject: String,
    pub(crate) name: String,
    pub(crate) groups: Vec<String>,
}

pub(crate) struct Oidc {
    client: ProfileClient,
}

impl Oidc {
    pub(crate) async fn discover(
        config: &Config,
        http: &HttpClient,
    ) -> Result<Self, Box<dyn Error>> {
        let metadata = CoreProviderMetadata::discover_async(
            IssuerUrl::from_url(config.oidc_issuer_url.clone()),
            http,
        )
        .await?;
        let redirect = RedirectUrl::new(format!(
            "{}/auth/callback",
            config.public_url.as_str().trim_end_matches('/')
        ))?;
        let client = ProfileClient::from_provider_metadata(
            metadata,
            ClientId::new(config.oidc_client_id.clone()),
            Some(ClientSecret::new(config.oidc_client_secret.clone())),
        )
        .set_redirect_uri(redirect);
        Ok(Self { client })
    }

    pub(crate) fn authorization(&self) -> (Url, String, LoginAttempt) {
        let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
        let (url, state, nonce) = self
            .client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scope(Scope::new("profile".to_owned()))
            .add_scope(Scope::new("groups".to_owned()))
            .set_pkce_challenge(challenge)
            .url();
        let attempt = LoginAttempt {
            nonce: nonce.secret().clone(),
            pkce_verifier: verifier.secret().clone(),
        };
        (url, state.secret().clone(), attempt)
    }

    pub(crate) async fn identity(
        &self,
        http: &HttpClient,
        code: String,
        attempt: LoginAttempt,
    ) -> Option<Identity> {
        let response = self
            .client
            .exchange_code(AuthorizationCode::new(code))
            .ok()?
            .set_pkce_verifier(PkceCodeVerifier::new(attempt.pkce_verifier))
            .request_async(http)
            .await
            .ok()?;
        let id_token = response.id_token()?;
        let claims = id_token
            .claims(&self.client.id_token_verifier(), &Nonce::new(attempt.nonce))
            .ok()?;
        let subject = claims.subject().as_str().to_owned();
        let profile = claims.additional_claims();
        let name = profile
            .name
            .as_deref()
            .or(profile.preferred_username.as_deref())
            .unwrap_or(&subject)
            .to_owned();
        let groups = profile.groups.clone().unwrap_or_default();
        Some(Identity {
            subject,
            name,
            groups,
        })
    }
}

/// Builds the HTTP client for the provider. Redirects are not followed, so requests stay on the
/// provider's own URLs.
pub(crate) fn http_client() -> Result<HttpClient, reqwest::Error> {
    reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(10))
        .build()
}
