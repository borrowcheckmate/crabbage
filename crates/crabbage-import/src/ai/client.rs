use std::fmt;
use std::time::Duration;

use reqwest::header::CONTENT_TYPE;
use reqwest::{Client, Response};
use serde_json::Value;
use url::Url;

use super::{AiDraft, AiError, Prompt, draft, request};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

/// A client for an OpenAI-compatible chat completions endpoint.
pub struct AiClient {
    base_url: Url,
    api_key: Option<String>,
    model: String,
    timeout: Duration,
}

impl fmt::Debug for AiClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AiClient")
            .field("base_url", &self.base_url.as_str())
            .field("api_key", &self.api_key.as_ref().map(|_| "<redacted>"))
            .field("model", &self.model)
            .field("timeout", &self.timeout)
            .finish()
    }
}

impl AiClient {
    /// A client for the endpoint at `base_url` (such as `https://host/v1`), sending `api_key` as a
    /// bearer token when it is set.
    #[must_use]
    pub const fn new(base_url: Url, api_key: Option<String>, model: String) -> Self {
        Self {
            base_url,
            api_key,
            model,
            timeout: DEFAULT_TIMEOUT,
        }
    }

    /// The same client with a different time limit for the whole request.
    #[must_use]
    pub fn with_timeout(self, timeout: Duration) -> Self {
        Self { timeout, ..self }
    }

    /// Sends the prompt and reads the draft from the first answer.
    ///
    /// # Errors
    ///
    /// [`AiError::Timeout`], [`AiError::Status`] for a non-success answer, [`AiError::Network`] for
    /// other failures, and [`AiError::Invalid`] when the answer has no usable draft.
    pub async fn complete(&self, prompt: &Prompt) -> Result<AiDraft, AiError> {
        let client = Client::builder()
            .timeout(self.timeout)
            .build()
            .map_err(|error| AiError::Network(error.without_url().to_string()))?;
        let mut builder = client
            .post(self.endpoint())
            .header(CONTENT_TYPE, "application/json")
            .body(request::body(&self.model, prompt).to_string());
        if let Some(key) = &self.api_key {
            builder = builder.bearer_auth(key);
        }
        let response = builder.send().await.map_err(failure)?;
        let content = message_content(response).await?;
        draft::parse(&content)
    }

    fn endpoint(&self) -> String {
        format!(
            "{}/chat/completions",
            self.base_url.as_str().trim_end_matches('/')
        )
    }
}

async fn message_content(response: Response) -> Result<String, AiError> {
    if !response.status().is_success() {
        return Err(AiError::Status(response.status().as_u16()));
    }
    let bytes = response.bytes().await.map_err(failure)?;
    serde_json::from_slice::<Value>(&bytes)
        .ok()
        .as_ref()
        .and_then(|body| body.pointer("/choices/0/message/content"))
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| AiError::Invalid("the response has no message".to_owned()))
}

fn failure(error: reqwest::Error) -> AiError {
    if error.is_timeout() {
        AiError::Timeout
    } else {
        AiError::Network(error.without_url().to_string())
    }
}
