//! Mealie API client tests.

mod client;

use crabbage_import::mealie::{MealieClient, MealieError};
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

#[expect(
    clippy::expect_used,
    reason = "the loopback listener must bind for tests"
)]
async fn bind() -> (TcpListener, String) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let address = listener.local_addr().expect("address");
    (listener, format!("http://{address}"))
}

type Handler = std::sync::Arc<dyn Fn(&str) -> (u16, String) + Send + Sync>;

async fn serve<F>(handler: F) -> (String, UnboundedReceiver<String>)
where
    F: Fn(&str) -> (u16, String) + Send + Sync + 'static,
{
    let (listener, base) = bind().await;
    let handler: Handler = std::sync::Arc::new(handler);
    let (requests, received) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(answer(
                stream,
                std::sync::Arc::clone(&handler),
                requests.clone(),
            ));
        }
    });
    (base, received)
}

async fn answer(mut stream: TcpStream, handler: Handler, requests: UnboundedSender<String>) {
    let request = read_request(&mut stream).await;
    drop(requests.send(request.clone()));
    let (status, body) = handler(&request);
    let reason = if status == 200 {
        "OK"
    } else if status == 401 {
        "Unauthorized"
    } else if status == 404 {
        "Not Found"
    } else {
        "Error"
    };
    let response = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    drop(stream.write_all(response.as_bytes()).await);
}

async fn read_request(stream: &mut TcpStream) -> String {
    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 1024];
    loop {
        let text = String::from_utf8_lossy(&bytes).into_owned();
        if text.contains("\r\n\r\n") {
            return text;
        }
        match stream.read(&mut chunk).await {
            Ok(0) | Err(_) => return text,
            Ok(length) => bytes.extend_from_slice(chunk.get(..length).unwrap_or_default()),
        }
    }
}

#[expect(clippy::expect_used, reason = "test URLs are valid loopback URLs")]
fn client(base: &str) -> MealieClient {
    MealieClient::new(base, "mealie-test-token").expect("client")
}

fn paged(request: &str) -> (u16, String) {
    let page = request
        .split("page=")
        .nth(1)
        .and_then(|part| part.split('&').next())
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(1);
    let entity = if request.contains("/api/recipes?") {
        "recipe"
    } else if request.contains("/api/foods?") {
        "food"
    } else if request.contains("/api/units?") {
        "unit"
    } else if request.contains("/api/organizers/categories?") {
        "category"
    } else {
        "tag"
    };
    let item = match entity {
        "recipe" => {
            json!({"id": format!("r{page}"), "slug": format!("slug{page}"), "name": format!("Recipe {page}")})
        }
        "food" => {
            json!({"id": format!("f{page}"), "name": format!("Food {page}"), "aliases": [{"name":"alias"}]})
        }
        "unit" => {
            json!({"id": format!("u{page}"), "name":"spoon", "abbreviation":"sp", "plural_name":"spoons"})
        }
        "category" => {
            json!({"id": format!("c{page}"), "name":"Dinner", "slug":format!("dinner{page}")})
        }
        _ => json!({"id": format!("t{page}"), "name":"Quick", "slug":format!("quick{page}")}),
    };
    (
        200,
        json!({"items":[item],"page":page,"per_page":1,"total":3,"total_pages":3}).to_string(),
    )
}

#[expect(clippy::expect_used, reason = "the test expects an API error")]
async fn error_of(status: u16, body: &'static str) -> MealieError {
    let (base, _) = serve(move |_| (status, body.to_owned())).await;
    client(&base).recipes().await.expect_err("error")
}
