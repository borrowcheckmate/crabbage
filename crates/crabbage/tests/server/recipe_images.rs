//! Recipe photos over HTTP: uploads are checked by their first bytes and stored under generated names,
//! only the users who may edit a recipe may change its photo, the photos are served to signed-in users
//! only, and a photo is removed with its recipe, or when it is replaced or deleted.

use axum::body::Body;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE, LOCATION};
use axum::http::{Request, Response, StatusCode};
use crabbage_store::recipes::{add_editor, get};
use http_body_util::BodyExt;

use crate::common::{Harness, SESSION_COOKIE, body_text, header, sign_in_as};
use crate::recipes::create;

const BOUNDARY: &str = "crabbage-test-boundary";
const OWNER: &str = "owner";
const OTHER: &str = "other";
const EDITOR: &str = "editor";
const JPEG: &[u8] = b"\xFF\xD8\xFF\xE0jpeg";
const PNG: &[u8] = b"\x89PNG\r\n\x1a\npng";
const WEBP: &[u8] = b"RIFF\x00\x00\x00\x00WEBPVP8 ";
const GIF: &[u8] = b"GIF89a\x01\x00\x01\x00";
const MAX_BYTES: usize = 10 * 1024 * 1024;

/// A multipart body with one file in field `field`, built by hand with the fixed boundary.
fn multipart(field: &str, filename: &str, content_type: &str, bytes: &[u8]) -> Vec<u8> {
    let head = format!(
        "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"{field}\"; filename=\"{filename}\"\r\nContent-Type: {content_type}\r\n\r\n"
    );
    let mut body = head.into_bytes();
    body.extend_from_slice(bytes);
    body.extend_from_slice(format!("\r\n--{BOUNDARY}--\r\n").as_bytes());
    body
}

/// Posts `body` as a multipart upload to recipe `id`, with the session `token` when given.
async fn upload(harness: &Harness, token: Option<&str>, id: i64, body: Vec<u8>) -> Response<Body> {
    let mut builder = Request::builder()
        .method("POST")
        .uri(format!("/recipes/{id}/image"))
        .header(
            "content-type",
            format!("multipart/form-data; boundary={BOUNDARY}"),
        );
    if let Some(token) = token {
        builder = builder.header("cookie", format!("{SESSION_COOKIE}={token}"));
    }
    harness
        .send(builder.body(Body::from(body)).expect("request builds"))
        .await
}

/// A valid upload of `bytes` as `image`, named `photo.jpg`.
async fn send_image(
    harness: &Harness,
    token: Option<&str>,
    id: i64,
    bytes: &[u8],
) -> Response<Body> {
    let body = multipart("image", "photo.jpg", "image/jpeg", bytes);
    upload(harness, token, id, body).await
}

/// The names of the files in the images directory, empty when it does not exist.
fn stored(harness: &Harness) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(harness.images_dir()) else {
        return Vec::new();
    };
    entries
        .map(|entry| {
            entry
                .expect("entry reads")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

/// Whether `name` is 32 lowercase hex characters, a dot and the extension `extension`.
fn is_generated(name: &str, extension: &str) -> bool {
    let Some((stem, found)) = name.split_once('.') else {
        return false;
    };
    found == extension
        && stem.len() == 32
        && stem
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

async fn photo_of(harness: &Harness, id: i64) -> Option<String> {
    get(&harness.pool, id)
        .await
        .expect("recipe reads")
        .expect("recipe exists")
        .image
}

async fn user_id(harness: &Harness, subject: &str) -> i64 {
    sqlx::query_scalar("SELECT id FROM users WHERE subject = ?")
        .bind(subject)
        .fetch_one(&harness.pool)
        .await
        .expect("the user exists")
}

async fn serve(harness: &Harness, token: Option<&str>, name: &str) -> Response<Body> {
    harness.get(&format!("/images/{name}"), token).await
}

async fn bytes_of(response: Response<Body>) -> Vec<u8> {
    response
        .into_body()
        .collect()
        .await
        .expect("body reads")
        .to_bytes()
        .to_vec()
}

#[tokio::test]
async fn uploads_of_each_type_are_stored_under_generated_names() {
    let harness = Harness::start("recipe-images-store").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    for (bytes, extension) in [(JPEG, "jpg"), (PNG, "png"), (WEBP, "webp")] {
        let id = create(&harness, &owner, "Pancakes").await;
        let response = send_image(&harness, Some(&owner), id, bytes).await;
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            header(&response, LOCATION).as_deref(),
            Some(format!("/recipes/{id}/edit").as_str())
        );
        let name = photo_of(&harness, id)
            .await
            .expect("the recipe has a photo");
        assert!(is_generated(&name, extension), "{name}");
        let written = std::fs::read(harness.images_dir().join(&name)).expect("file exists");
        assert_eq!(written, bytes);
    }
    assert_eq!(stored(&harness).len(), 3);
    assert!(stored(&harness).iter().all(|name| !name.starts_with('.')));
}

#[tokio::test]
async fn served_images_carry_their_type_and_a_cache_header_for_signed_in_users() {
    let harness = Harness::start("recipe-images-serve").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    for (bytes, mime) in [
        (JPEG, "image/jpeg"),
        (PNG, "image/png"),
        (WEBP, "image/webp"),
    ] {
        let id = create(&harness, &owner, "Pancakes").await;
        send_image(&harness, Some(&owner), id, bytes).await;
        let name = photo_of(&harness, id)
            .await
            .expect("the recipe has a photo");
        let response = serve(&harness, Some(&owner), &name).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(header(&response, CONTENT_TYPE).as_deref(), Some(mime));
        assert_eq!(
            header(&response, CACHE_CONTROL).as_deref(),
            Some("private, max-age=31536000, immutable")
        );
        assert_eq!(bytes_of(response).await, bytes);
    }
}

#[tokio::test]
async fn images_are_not_served_without_a_session() {
    let harness = Harness::start("recipe-images-anonymous").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &owner, "Pancakes").await;
    send_image(&harness, Some(&owner), id, JPEG).await;
    let name = photo_of(&harness, id)
        .await
        .expect("the recipe has a photo");

    let response = serve(&harness, None, &name).await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(header(&response, LOCATION).as_deref(), Some("/auth/login"));
    assert!(!body_text(response).await.contains("jpeg"));
}

#[tokio::test]
async fn unsupported_files_are_refused_and_nothing_is_written() {
    let harness = Harness::start("recipe-images-refused").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &owner, "Pancakes").await;

    let gif = send_image(&harness, Some(&owner), id, GIF).await;
    assert_eq!(gif.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    let text = multipart("image", "photo.jpg", "image/jpeg", b"not a picture");
    let text = upload(&harness, Some(&owner), id, text).await;
    assert_eq!(text.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    let other_field = multipart("photo", "photo.jpg", "image/jpeg", JPEG);
    let other_field = upload(&harness, Some(&owner), id, other_field).await;
    assert_eq!(other_field.status(), StatusCode::BAD_REQUEST);

    assert_eq!(stored(&harness), Vec::<String>::new());
    assert_eq!(photo_of(&harness, id).await, None);
}

#[tokio::test]
async fn an_upload_over_the_limit_is_too_large() {
    let harness = Harness::start("recipe-images-large").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &owner, "Pancakes").await;
    let mut bytes = PNG.to_vec();
    bytes.resize(MAX_BYTES + 1, 0);

    let response = send_image(&harness, Some(&owner), id, &bytes).await;
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(stored(&harness), Vec::<String>::new());
    assert_eq!(photo_of(&harness, id).await, None);
}

#[tokio::test]
async fn only_editors_may_change_a_photo_and_signed_out_users_are_sent_to_sign_in() {
    let harness = Harness::start("recipe-images-rights").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let other = sign_in_as(&harness, OTHER, &[]).await;
    let editor = sign_in_as(&harness, EDITOR, &[]).await;
    let id = create(&harness, &owner, "Pancakes").await;
    let editor_id = user_id(&harness, EDITOR).await;
    add_editor(&harness.pool, id, editor_id)
        .await
        .expect("editor added");
    send_image(&harness, Some(&owner), id, JPEG).await;
    let before = photo_of(&harness, id).await;

    let refused = send_image(&harness, Some(&other), id, PNG).await;
    assert_eq!(refused.status(), StatusCode::FORBIDDEN);
    let removed = harness
        .post_form(&format!("/recipes/{id}/image/delete"), Some(&other), "")
        .await;
    assert_eq!(removed.status(), StatusCode::FORBIDDEN);
    assert_eq!(photo_of(&harness, id).await, before);

    let allowed = send_image(&harness, Some(&editor), id, PNG).await;
    assert_eq!(allowed.status(), StatusCode::SEE_OTHER);

    let signed_out = send_image(&harness, None, id, WEBP).await;
    assert_eq!(signed_out.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        header(&signed_out, LOCATION).as_deref(),
        Some("/auth/login")
    );

    let missing = send_image(&harness, Some(&owner), 999, JPEG).await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn bad_image_names_are_not_found() {
    let harness = Harness::start("recipe-images-names").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &owner, "Pancakes").await;
    send_image(&harness, Some(&owner), id, JPEG).await;
    let name = photo_of(&harness, id)
        .await
        .expect("the recipe has a photo");
    let paths = [
        "/images/..%2Fx".to_owned(),
        "/images/%2e%2e%2f1234".to_owned(),
        "/images/abc.jpg".to_owned(),
        format!("/images/{}", name.to_uppercase()),
        format!("/images/{}", name.replace(".jpg", ".gif")),
        "/images/0123456789abcdef0123456789abcdef.jpg".to_owned(),
    ];
    for path in paths {
        let response = harness.get(&path, Some(&owner)).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{path}");
    }
}

#[tokio::test]
async fn replacing_a_photo_removes_the_old_file() {
    let harness = Harness::start("recipe-images-replace").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &owner, "Pancakes").await;
    send_image(&harness, Some(&owner), id, PNG).await;
    let first = photo_of(&harness, id)
        .await
        .expect("the recipe has a photo");

    send_image(&harness, Some(&owner), id, JPEG).await;
    let second = photo_of(&harness, id)
        .await
        .expect("the recipe has a photo");

    assert_ne!(first, second);
    assert!(!harness.images_dir().join(&first).exists());
    assert_eq!(stored(&harness), vec![second]);
}

#[tokio::test]
async fn removing_a_photo_deletes_the_file_and_clears_it() {
    let harness = Harness::start("recipe-images-remove").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &owner, "Pancakes").await;
    send_image(&harness, Some(&owner), id, JPEG).await;
    let name = photo_of(&harness, id)
        .await
        .expect("the recipe has a photo");

    let response = harness
        .post_form(&format!("/recipes/{id}/image/delete"), Some(&owner), "")
        .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        header(&response, LOCATION).as_deref(),
        Some(format!("/recipes/{id}/edit").as_str())
    );
    assert!(!harness.images_dir().join(&name).exists());
    assert_eq!(photo_of(&harness, id).await, None);

    let again = harness
        .post_form(&format!("/recipes/{id}/image/delete"), Some(&owner), "")
        .await;
    assert_eq!(again.status(), StatusCode::SEE_OTHER);
}

#[tokio::test]
async fn deleting_a_recipe_deletes_its_photo() {
    let harness = Harness::start("recipe-images-recipe-delete").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &owner, "Pancakes").await;
    send_image(&harness, Some(&owner), id, PNG).await;
    let name = photo_of(&harness, id)
        .await
        .expect("the recipe has a photo");

    let response = harness
        .post_form("/api/delete_recipe", Some(&owner), &format!("id={id}"))
        .await;
    let status = response.status();
    assert_eq!(status, StatusCode::OK, "{}", body_text(response).await);
    assert!(get(&harness.pool, id).await.expect("reads").is_none());
    assert!(!harness.images_dir().join(&name).exists());
}

#[tokio::test]
async fn the_recipe_page_shows_the_photo_and_the_edit_page_its_card() {
    let harness = Harness::start("recipe-images-pages").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &owner, "Pancakes").await;

    let bare = body_text(
        harness
            .get(&format!("/recipes/{id}/edit"), Some(&owner))
            .await,
    )
    .await;
    assert!(
        bare.contains(&format!("action=\"/recipes/{id}/image\"")),
        "{bare}"
    );
    assert!(!bare.contains("/image/delete"), "{bare}");

    send_image(&harness, Some(&owner), id, JPEG).await;
    let name = photo_of(&harness, id)
        .await
        .expect("the recipe has a photo");
    let detail = body_text(harness.get(&format!("/recipes/{id}"), Some(&owner)).await).await;
    assert!(
        detail.contains(&format!("src=\"/images/{name}\"")),
        "{detail}"
    );
    assert!(detail.contains("alt=\"Pancakes\""), "{detail}");

    let edit = body_text(
        harness
            .get(&format!("/recipes/{id}/edit"), Some(&owner))
            .await,
    )
    .await;
    assert!(edit.contains(&format!("src=\"/images/{name}\"")), "{edit}");
    assert!(
        edit.contains(&format!("action=\"/recipes/{id}/image/delete\"")),
        "{edit}"
    );
    assert!(
        edit.contains("accept=\"image/jpeg,image/png,image/webp\""),
        "{edit}"
    );
}
