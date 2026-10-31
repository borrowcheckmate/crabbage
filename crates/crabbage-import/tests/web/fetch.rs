use std::time::Duration;

use crabbage_import::web::{ImportError, Limits, fetch, fetch_bytes, import};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const PLAIN: &str = include_str!("../fixtures/web/plain.html");

fn open() -> Limits {
    Limits {
        allow_private: true,
        ..Limits::default()
    }
}

#[expect(
    clippy::expect_used,
    reason = "the loopback listener must bind for the tests to run"
)]
async fn serve() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let address = listener.local_addr().expect("address");
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(answer(stream));
        }
    });
    format!("http://{address}")
}

async fn answer(mut stream: TcpStream) {
    let mut request = Vec::new();
    let mut chunk = [0_u8; 1024];
    while !request.ends_with(b"\r\n\r\n") {
        match stream.read(&mut chunk).await {
            Ok(0) | Err(_) => return,
            Ok(read) => request.extend_from_slice(chunk.get(..read).unwrap_or_default()),
        }
    }
    let text = String::from_utf8_lossy(&request).into_owned();
    let path = text.split_whitespace().nth(1).unwrap_or("/").to_owned();
    let response = match path.as_str() {
        "/ok" => sized(PLAIN),
        "/typed" => "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: 3\r\nConnection: close\r\n\r\n\x01\x02\x03".to_owned(),
        "/big" => sized(&"x".repeat(4000)),
        "/unsized" => format!(
            "HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n{}",
            "x".repeat(4000)
        ),
        "/missing" => {
            "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_owned()
        }
        "/redirect" => {
            "HTTP/1.1 302 Found\r\nLocation: /ok\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                .to_owned()
        }
        _ => std::future::pending().await,
    };
    drop(stream.write_all(response.as_bytes()).await);
}

fn sized(body: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

#[tokio::test]
async fn a_page_is_returned_as_text() {
    let base = serve().await;
    let body = fetch(&format!("{base}/ok"), &open()).await.expect("page");
    assert_eq!(body, PLAIN);
}

#[tokio::test]
async fn a_redirect_to_another_path_is_followed() {
    let base = serve().await;
    let body = fetch(&format!("{base}/redirect"), &open())
        .await
        .expect("redirected page");
    assert_eq!(body, PLAIN);
}

#[tokio::test]
async fn a_declared_body_over_the_limit_is_too_large() {
    let base = serve().await;
    let limits = Limits {
        max_bytes: 1000,
        ..open()
    };
    assert_eq!(
        fetch(&format!("{base}/big"), &limits).await,
        Err(ImportError::TooLarge)
    );
}

#[tokio::test]
async fn an_undeclared_body_over_the_limit_is_too_large() {
    let base = serve().await;
    let limits = Limits {
        max_bytes: 1000,
        ..open()
    };
    assert_eq!(
        fetch(&format!("{base}/unsized"), &limits).await,
        Err(ImportError::TooLarge)
    );
}

#[tokio::test]
async fn a_server_that_never_answers_times_out() {
    let base = serve().await;
    let limits = Limits {
        timeout: Duration::from_millis(200),
        ..open()
    };
    assert_eq!(
        fetch(&format!("{base}/hang"), &limits).await,
        Err(ImportError::Timeout)
    );
}

#[tokio::test]
async fn an_error_status_is_reported() {
    let base = serve().await;
    assert_eq!(
        fetch(&format!("{base}/missing"), &open()).await,
        Err(ImportError::Status(404))
    );
}

#[tokio::test]
async fn import_reads_the_recipe_from_a_fetched_page() {
    let base = serve().await;
    let draft = import(&format!("{base}/redirect"), &open())
        .await
        .expect("imported recipe");
    assert_eq!(draft.title, "Lemon Pancakes");
}

#[tokio::test]
async fn non_http_urls_are_refused() {
    for url in [
        "file:///etc/passwd",
        "ftp://example.com/",
        "javascript:alert(1)",
        "not a url",
    ] {
        assert_eq!(
            fetch(url, &Limits::default()).await,
            Err(ImportError::NotHttp),
            "{url}"
        );
    }
}

#[tokio::test]
async fn private_addresses_are_refused_by_default() {
    for url in [
        "http://127.0.0.1/",
        "http://10.0.0.1/",
        "http://192.168.1.1/",
        "http://169.254.169.254/",
        "http://[::1]/",
        "http://localhost/",
    ] {
        assert_eq!(
            fetch(url, &Limits::default()).await,
            Err(ImportError::PrivateAddress),
            "{url}"
        );
    }
}

#[tokio::test]
async fn bytes_come_back_with_the_declared_content_type_and_follow_the_same_checks() {
    let base = serve().await;
    let fetched = fetch_bytes(&format!("{base}/typed"), &open())
        .await
        .expect("bytes");
    assert_eq!(fetched.bytes, vec![1, 2, 3]);
    assert_eq!(fetched.content_type.as_deref(), Some("image/png"));
    let untyped = fetch_bytes(&format!("{base}/ok"), &open())
        .await
        .expect("page");
    assert_eq!(untyped.content_type, None);
    assert_eq!(
        fetch_bytes(&format!("{base}/ok"), &Limits::default()).await,
        Err(ImportError::PrivateAddress)
    );
}
