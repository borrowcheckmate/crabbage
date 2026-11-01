use crabbage_import::ai::{AiClient, Image};
use crabbage_import::pipeline::from_photos;
use serde_json::Value;
use url::Url;

use crate::candidate;
use crate::server::{VALID, completion, serve};

#[expect(clippy::expect_used, reason = "the test client URL is always valid")]
fn client(base: &str) -> AiClient {
    AiClient::new(
        Url::parse(&format!("{base}/v1")).expect("url"),
        None,
        "test-model".to_owned(),
    )
}

fn image(bytes: &[u8]) -> Image {
    Image {
        media_type: "image/png".to_owned(),
        bytes: bytes.to_vec(),
    }
}

#[expect(
    clippy::expect_used,
    reason = "a request without a JSON body is a failed test"
)]
fn body_of(request: &str) -> Value {
    let (_, body) = request.split_once("\r\n\r\n").expect("a body");
    serde_json::from_str(body).expect("a JSON body")
}

#[tokio::test]
async fn every_photo_is_sent_and_the_answer_becomes_the_recipe() {
    let (base, mut requests) = serve("200 OK", &completion(VALID)).await;
    let flour = candidate(1, "flour");
    let photos = vec![image(b"one"), image(b"two")];
    let result = from_photos(photos, &[flour], &[], &client(&base), "nl")
        .await
        .expect("a recipe");
    assert_eq!(result.title, "Fluffy pancakes");
    assert_eq!(result.image, None);
    assert_eq!(result.lines.len(), 2);
    assert_eq!(result.unmatched, vec!["eggs".to_owned()]);
    let request = requests.try_recv().expect("one request");
    assert!(requests.try_recv().is_err());
    let body = body_of(&request);
    let parts = body["messages"][1]["content"].as_array().expect("parts");
    let urls: Vec<&str> = parts
        .iter()
        .filter_map(|part| part["image_url"]["url"].as_str())
        .collect();
    assert_eq!(urls.len(), 2, "{parts:?}");
    assert!(
        urls.iter()
            .all(|url| url.starts_with("data:image/png;base64,"))
    );
    let instructions = body["messages"][0]["content"].as_str().unwrap_or_default();
    assert!(
        instructions.contains("language with the code nl"),
        "{instructions}"
    );
    assert!(instructions.contains("from the photos"), "{instructions}");
}

#[tokio::test]
async fn an_ai_error_is_returned() {
    let (base, _requests) = serve("500 Internal Server Error", "{}").await;
    let result = from_photos(vec![image(b"one")], &[], &[], &client(&base), "en").await;
    assert_eq!(
        result.expect_err("an error").to_string(),
        "the AI service answered with status 500"
    );
}
