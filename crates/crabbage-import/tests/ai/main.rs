//! AI client: the request sent to the chat endpoint, the draft read from its answer, error
//! messages, and that the API key never shows up in them. Every server is on loopback.

mod client;
mod errors;
mod parse;

use crabbage_import::ai::{AiClient, AiError, Image, Prompt};
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use url::Url;

const KEY: &str = "sk-super-secret-123";
const PNG: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
const VALID: &str = r#"{"title":"Lemon Pancakes","servings":2,"prep_minutes":5,"cook_minutes":10,"lines":[{"amount":200,"unit":"g","name":"flour","note":null,"ingredient_id":null},{"amount":1,"unit":"piece","name":"egg"}],"steps":[{"text":"Whisk flour and egg.","lines":[0,1]}]}"#;

fn prompt(text: &str) -> Prompt {
    Prompt {
        instructions: "Read the recipe.".to_owned(),
        text: text.to_owned(),
        images: vec![Image {
            media_type: "image/png".to_owned(),
            bytes: PNG.to_vec(),
        }],
    }
}

#[expect(clippy::expect_used, reason = "the test client URL is always valid")]
fn ai(base: &str, key: Option<&str>) -> AiClient {
    AiClient::new(
        Url::parse(base).expect("url"),
        key.map(str::to_owned),
        "test-model".to_owned(),
    )
}

fn completion(content: &str) -> String {
    json!({"choices": [{"message": {"content": content}}]}).to_string()
}

#[expect(
    clippy::expect_used,
    reason = "the loopback listener must bind for the tests to run"
)]
async fn bind() -> (TcpListener, String) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let address = listener.local_addr().expect("address");
    (listener, format!("http://{address}"))
}

/// A server that answers every request with `status` and `body`, and reports each raw request.
async fn serve(status: &str, body: &str) -> (String, UnboundedReceiver<String>) {
    let (listener, base) = bind().await;
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let (requests, received) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(answer(stream, response.clone(), requests.clone()));
        }
    });
    (base, received)
}

async fn answer(mut stream: TcpStream, response: String, requests: UnboundedSender<String>) {
    let request = read_request(&mut stream).await;
    drop(requests.send(request));
    drop(stream.write_all(response.as_bytes()).await);
}

/// Reads the headers and the body announced by `Content-Length`.
async fn read_request(stream: &mut TcpStream) -> String {
    let mut request = Vec::new();
    let mut chunk = [0_u8; 1024];
    loop {
        let text = String::from_utf8_lossy(&request).into_owned();
        if let Some((head, body)) = text.split_once("\r\n\r\n") {
            let length = head
                .lines()
                .find_map(|line| header_value(line, "content-length"))
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(0);
            if body.len() >= length {
                return text;
            }
        }
        match stream.read(&mut chunk).await {
            Ok(0) | Err(_) => return text,
            Ok(read) => request.extend_from_slice(chunk.get(..read).unwrap_or_default()),
        }
    }
}

/// A server that reads the request and never answers.
async fn silent() -> String {
    let (listener, base) = bind().await;
    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                drop(read_request(&mut stream).await);
                std::future::pending::<()>().await;
            });
        }
    });
    base
}

/// An address with nothing listening on it.
#[expect(
    clippy::expect_used,
    reason = "the loopback listener must bind for the tests to run"
)]
async fn refused() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let address = listener.local_addr().expect("address");
    drop(listener);
    format!("http://{address}")
}

fn header_value<'a>(line: &'a str, name: &str) -> Option<&'a str> {
    let (key, value) = line.split_once(':')?;
    key.eq_ignore_ascii_case(name).then(|| value.trim())
}

fn request_path(request: &str) -> &str {
    request.split_whitespace().nth(1).unwrap_or_default()
}

fn header<'a>(request: &'a str, name: &str) -> Option<&'a str> {
    request.lines().find_map(|line| header_value(line, name))
}

#[expect(
    clippy::expect_used,
    reason = "a request without a JSON body is a failed test"
)]
fn body_json(request: &str) -> Value {
    let (_, body) = request.split_once("\r\n\r\n").expect("a body");
    serde_json::from_str(body).expect("a JSON body")
}

/// The error of one call to a server that answers with `status` and `body`.
#[expect(clippy::expect_used, reason = "a call that succeeds is a failed test")]
async fn failure_of(status: &str, body: &str) -> AiError {
    let (base, _) = serve(status, body).await;
    ai(&format!("{base}/v1"), Some(KEY))
        .complete(&prompt("Lemon pancakes"))
        .await
        .expect_err("an error")
}
