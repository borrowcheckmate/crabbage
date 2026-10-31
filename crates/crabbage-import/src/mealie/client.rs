use std::time::Duration;

use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use reqwest::{Client, Response, StatusCode};
use serde::de::DeserializeOwned;
use url::Url;

use super::model::Paginated;
use super::{Category, Food, MealieError, MealieUnit, Recipe, RecipeSummary, Tag};

const PAGE_SIZE: usize = 100;
const MAX_BYTES: usize = 5 * 1024 * 1024;
const TIMEOUT: Duration = Duration::from_secs(10);

/// Reads recipes and catalog data from a Mealie API.
#[derive(Debug)]
pub struct MealieClient {
    base_url: Url,
    client: Client,
}

impl MealieClient {
    /// Creates a Mealie API client using `token` as a bearer token.
    ///
    /// # Errors
    ///
    /// Returns [`MealieError::InvalidUrl`] if `base_url` is not an HTTP or HTTPS URL with a host,
    /// or [`MealieError::Network`] if the HTTP client cannot be built.
    pub fn new(base_url: &str, token: &str) -> Result<Self, MealieError> {
        let mut base_url = Url::parse(base_url).map_err(|_error| MealieError::InvalidUrl)?;
        if !matches!(base_url.scheme(), "http" | "https") || base_url.host_str().is_none() {
            return Err(MealieError::InvalidUrl);
        }
        base_url.set_query(None);
        base_url.set_fragment(None);
        let mut headers = HeaderMap::new();
        let mut authorization = HeaderValue::from_str(&format!("Bearer {token}"))
            .map_err(|_error| MealieError::Network("invalid API token header".to_owned()))?;
        authorization.set_sensitive(true);
        headers.insert(AUTHORIZATION, authorization);
        let client = Client::builder()
            .default_headers(headers)
            .timeout(TIMEOUT)
            .user_agent("crabbage")
            .build()
            .map_err(|error| MealieError::Network(error.without_url().to_string()))?;
        Ok(Self { base_url, client })
    }

    /// Lists recipe summaries from every page.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails, the token is refused, or the response is invalid.
    pub async fn recipes(&self) -> Result<Vec<RecipeSummary>, MealieError> {
        self.list("/api/recipes").await
    }

    /// Reads a recipe by its Mealie slug.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails, the token is refused, or the response is invalid.
    pub async fn recipe(&self, slug: &str) -> Result<Recipe, MealieError> {
        self.get_json(&format!("/api/recipes/{}", encode_segment(slug)))
            .await
    }

    /// Lists foods from every page.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails, the token is refused, or the response is invalid.
    pub async fn foods(&self) -> Result<Vec<Food>, MealieError> {
        self.list("/api/foods").await
    }

    /// Lists units from every page.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails, the token is refused, or the response is invalid.
    pub async fn units(&self) -> Result<Vec<MealieUnit>, MealieError> {
        self.list("/api/units").await
    }

    /// Lists recipe categories from every page.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails, the token is refused, or the response is invalid.
    pub async fn categories(&self) -> Result<Vec<Category>, MealieError> {
        self.list("/api/organizers/categories").await
    }

    /// Lists recipe tags from every page.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails, the token is refused, or the response is invalid.
    pub async fn tags(&self) -> Result<Vec<Tag>, MealieError> {
        self.list("/api/organizers/tags").await
    }

    /// Downloads the original WebP image for a recipe, up to 5 MiB.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails, the token is refused, or the response is invalid.
    pub async fn image(&self, recipe: &Recipe) -> Result<Vec<u8>, MealieError> {
        let response = self
            .client
            .get(self.endpoint(&format!(
                "/api/media/recipes/{}/images/original.webp",
                encode_segment(&recipe.id)
            )))
            .send()
            .await
            .map_err(network)?;
        let response = check_status(response)?;
        let mut bytes = Vec::new();
        let mut stream = response;
        if stream
            .content_length()
            .is_some_and(|size| size > u64::try_from(MAX_BYTES).unwrap_or(u64::MAX))
        {
            return Err(MealieError::TooLarge);
        }
        while let Some(chunk) = stream.chunk().await.map_err(network)? {
            if bytes.len().saturating_add(chunk.len()) > MAX_BYTES {
                return Err(MealieError::TooLarge);
            }
            bytes.extend_from_slice(&chunk);
        }
        Ok(bytes)
    }

    async fn list<T: DeserializeOwned>(&self, path: &str) -> Result<Vec<T>, MealieError> {
        let mut items = Vec::new();
        let mut page = 1;
        loop {
            let url = self.endpoint(path);
            let response = self
                .client
                .get(url)
                .query(&[("page", page), ("perPage", PAGE_SIZE)])
                .send()
                .await
                .map_err(network)?;
            let result: Paginated<T> = match parse_json(response).await {
                Err(MealieError::NotFound) => return Err(MealieError::NotMealie),
                result => result?,
            };
            if result.items.is_empty() {
                break;
            }
            let last = result.current.max(page) >= result.total_pages.max(1);
            items.extend(result.items);
            if last {
                break;
            }
            page += 1;
        }
        Ok(items)
    }

    async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, MealieError> {
        let response = self
            .client
            .get(self.endpoint(path))
            .send()
            .await
            .map_err(network)?;
        parse_json(response).await
    }

    fn endpoint(&self, path: &str) -> Url {
        let root = self.base_url.as_str().trim_end_matches('/');
        Url::parse(&format!("{root}{path}")).unwrap_or_else(|_| self.base_url.clone())
    }
}

fn encode_segment(segment: &str) -> String {
    segment
        .bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
                char::from(byte).to_string()
            } else {
                format!("%{byte:02X}")
            }
        })
        .collect()
}

async fn parse_json<T: DeserializeOwned>(response: Response) -> Result<T, MealieError> {
    let response = check_status(response)?;
    let bytes = response.bytes().await.map_err(network)?;
    serde_json::from_slice(&bytes).map_err(|_error| MealieError::NotMealie)
}

fn check_status(response: Response) -> Result<Response, MealieError> {
    match response.status() {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(MealieError::Unauthorized),
        StatusCode::NOT_FOUND => Err(MealieError::NotFound),
        status if !status.is_success() => Err(MealieError::Status(status.as_u16())),
        _ => Ok(response),
    }
}

fn network(error: reqwest::Error) -> MealieError {
    if error.is_timeout() {
        MealieError::Timeout
    } else {
        MealieError::Network(error.without_url().to_string())
    }
}
