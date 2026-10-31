use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

pub(crate) const VALID: &str = r#"{"title":"Fluffy pancakes","servings":2,"lines":[{"amount":200,"unit":"g","name":"flour","note":null,"ingredient_id":null},{"amount":2,"unit":"piece","name":"eggs","note":null,"ingredient_id":null}],"steps":[{"text":"Whisk.","lines":[0,1]}]}"#;

pub(crate) fn completion(content: &str) -> String {
    serde_json::json!({"choices": [{"message": {"content": content}}]}).to_string()
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

/// A server on loopback that answers every request with `status` and `body`, and reports each raw
/// request it receives.
pub(crate) async fn serve(status: &str, body: &str) -> (String, UnboundedReceiver<String>) {
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

async fn read_request(stream: &mut TcpStream) -> String {
    let mut request = Vec::new();
    let mut chunk = [0_u8; 1024];
    loop {
        let text = String::from_utf8_lossy(&request).into_owned();
        if let Some((head, body)) = text.split_once("\r\n\r\n") {
            let length = head
                .lines()
                .find_map(|line| {
                    let (key, value) = line.split_once(':')?;
                    key.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().ok())
                        .flatten()
                })
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
