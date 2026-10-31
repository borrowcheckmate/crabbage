use std::net::SocketAddr;
use std::time::Duration;

use reqwest::header::LOCATION;
use reqwest::redirect::Policy;
use reqwest::{Client, Response};
use tokio::net::lookup_host;
use url::Url;

use super::{ImportError, is_public};

const MAX_REDIRECTS: usize = 5;

/// What a fetch may take: the time, the size of the body, and whether private addresses are
/// allowed (only for tests on loopback).
#[derive(Debug, Clone)]
pub struct Limits {
    /// The time for the whole request, body included.
    pub timeout: Duration,
    /// The largest body accepted, in bytes.
    pub max_bytes: usize,
    /// Whether addresses that are not public may be fetched.
    pub allow_private: bool,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(10),
            max_bytes: 5 * 1024 * 1024,
            allow_private: false,
        }
    }
}

/// Fetches the page at `url` as UTF-8 text. Each hop is checked for scheme and address, and
/// connects to the address that was checked.
///
/// # Errors
///
/// [`ImportError::NotHttp`] for other schemes, [`ImportError::PrivateAddress`] for non-public
/// hosts, [`ImportError::Timeout`], [`ImportError::TooLarge`], [`ImportError::Status`] for a
/// non-success answer and [`ImportError::Network`] for other failures.
pub async fn fetch(url: &str, limits: &Limits) -> Result<String, ImportError> {
    let mut target = parse(url)?;
    let mut redirects = 0;
    loop {
        let address = checked_address(&target, limits.allow_private).await?;
        let host = host_of(&target)?;
        let client = pinned_client(&host, address, limits.timeout)?;
        let response = client
            .get(target.clone())
            .send()
            .await
            .map_err(|error| failure(&error))?;
        if let Some(location) = redirect_location(&response) {
            if redirects == MAX_REDIRECTS {
                return Err(ImportError::Network("too many redirects".to_owned()));
            }
            target = target
                .join(&location)
                .map_err(|error| ImportError::Network(error.to_string()))?;
            redirects += 1;
            continue;
        }
        return body(response, limits.max_bytes).await;
    }
}

fn parse(url: &str) -> Result<Url, ImportError> {
    let Ok(parsed) = Url::parse(url) else {
        return Err(ImportError::NotHttp);
    };
    if matches!(parsed.scheme(), "http" | "https") {
        Ok(parsed)
    } else {
        Err(ImportError::NotHttp)
    }
}

fn host_of(url: &Url) -> Result<String, ImportError> {
    url.host_str()
        .map(|host| {
            host.trim_start_matches('[')
                .trim_end_matches(']')
                .to_owned()
        })
        .ok_or(ImportError::NotHttp)
}

async fn checked_address(url: &Url, allow_private: bool) -> Result<SocketAddr, ImportError> {
    let host = host_of(url)?;
    let port = url.port_or_known_default().ok_or(ImportError::NotHttp)?;
    let addresses: Vec<SocketAddr> = lookup_host((host.as_str(), port))
        .await
        .map_err(|error| ImportError::Network(error.to_string()))?
        .collect();
    if !allow_private && addresses.iter().any(|address| !is_public(address.ip())) {
        return Err(ImportError::PrivateAddress);
    }
    addresses
        .first()
        .copied()
        .ok_or_else(|| ImportError::Network("the host has no address".to_owned()))
}

fn pinned_client(
    host: &str,
    address: SocketAddr,
    timeout: Duration,
) -> Result<Client, ImportError> {
    Client::builder()
        .no_proxy()
        .redirect(Policy::none())
        .timeout(timeout)
        .resolve(host, address)
        .build()
        .map_err(|error| ImportError::Network(error.to_string()))
}

fn redirect_location(response: &Response) -> Option<String> {
    if !response.status().is_redirection() {
        return None;
    }
    response
        .headers()
        .get(LOCATION)?
        .to_str()
        .ok()
        .map(str::to_owned)
}

async fn body(mut response: Response, max_bytes: usize) -> Result<String, ImportError> {
    if !response.status().is_success() {
        return Err(ImportError::Status(response.status().as_u16()));
    }
    let limit = u64::try_from(max_bytes).unwrap_or(u64::MAX);
    if response
        .content_length()
        .is_some_and(|length| length > limit)
    {
        return Err(ImportError::TooLarge);
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|error| failure(&error))? {
        if bytes.len().saturating_add(chunk.len()) > max_bytes {
            return Err(ImportError::TooLarge);
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn failure(error: &reqwest::Error) -> ImportError {
    if error.is_timeout() {
        ImportError::Timeout
    } else {
        ImportError::Network(error.to_string())
    }
}
