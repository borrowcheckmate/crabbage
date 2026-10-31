//! Ratings and favourites over HTTP: members rate and favourite any recipe they see, one rating per
//! user that changes in place, the average and count on the recipe and the list, favourites per user,
//! and the favourites filter on the list.

use axum::http::StatusCode;
use axum::http::header::LOCATION;

use crate::common::{Harness, body_text, header, sign_in_as};
use crate::recipes::create;

const OWNER: &str = "owner";
const FRIEND: &str = "friend";
const OTHER: &str = "other";

async fn page(harness: &Harness, token: &str, path: &str) -> String {
    let response = harness.get(path, Some(token)).await;
    assert_eq!(response.status(), StatusCode::OK, "{path}");
    body_text(response).await
}

async fn post(harness: &Harness, token: &str, path: &str, body: &str) -> StatusCode {
    harness.post_form(path, Some(token), body).await.status()
}

/// Whether the button whose `aria-label` is `label` is pressed, or `None` when there is no such
/// button.
fn pressed(body: &str, label: &str) -> Option<bool> {
    let needle = format!("aria-label=\"{label}\"");
    body.split("<button")
        .find(|button| button.contains(&needle))
        .and_then(|button| button.split('>').next())
        .map(|tag| tag.contains("aria-pressed=\"true\""))
}

#[tokio::test]
async fn two_users_rate_and_the_page_and_the_list_show_the_average_and_count() {
    let harness = Harness::start("ratings-average").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let friend = sign_in_as(&harness, FRIEND, &[]).await;
    let id = create(&harness, &owner, "Pancakes").await;
    let path = format!("/recipes/{id}");

    let body = page(&harness, &owner, &path).await;
    assert!(body.contains("No ratings yet"), "{body}");

    let response = harness
        .post_form(&format!("{path}/rating"), Some(&owner), "stars=4")
        .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(header(&response, LOCATION).as_deref(), Some(path.as_str()));
    assert_eq!(
        post(&harness, &friend, &format!("{path}/rating"), "stars=5").await,
        StatusCode::SEE_OTHER
    );

    let body = page(&harness, &owner, &path).await;
    assert!(body.contains("★ 4.5 (2 ratings)"), "{body}");
    assert!(!body.contains("No ratings yet"), "{body}");
    let list = page(&harness, &friend, "/recipes").await;
    assert!(list.contains("★ 4.5 (2 ratings)"), "{list}");
}

#[tokio::test]
async fn one_rating_is_singular_and_shows_the_viewers_own_star_pressed() {
    let harness = Harness::start("ratings-single").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let friend = sign_in_as(&harness, FRIEND, &[]).await;
    let id = create(&harness, &owner, "Pancakes").await;
    let path = format!("/recipes/{id}");
    post(&harness, &owner, &format!("{path}/rating"), "stars=3").await;

    let body = page(&harness, &owner, &path).await;
    assert!(body.contains("★ 3.0 (1 rating)"), "{body}");
    assert!(body.contains("Rate 3 out of 5"), "{body}");
    assert_eq!(pressed(&body, "Rate 3 out of 5"), Some(true), "{body}");
    assert_eq!(pressed(&body, "Rate 4 out of 5"), Some(false), "{body}");

    let body = page(&harness, &friend, &path).await;
    assert!(body.contains("Rate 3 out of 5"), "{body}");
    assert_eq!(pressed(&body, "Rate 3 out of 5"), Some(false), "{body}");
}

#[tokio::test]
async fn changing_a_rating_replaces_it() {
    let harness = Harness::start("ratings-change").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &owner, "Pancakes").await;
    let path = format!("/recipes/{id}/rating");
    post(&harness, &owner, &path, "stars=4").await;
    post(&harness, &owner, &path, "stars=2").await;

    let body = page(&harness, &owner, &format!("/recipes/{id}")).await;
    assert!(body.contains("★ 2.0 (1 rating)"), "{body}");
    assert!(!body.contains("★ 4.0"), "{body}");
    assert_eq!(pressed(&body, "Rate 2 out of 5"), Some(true), "{body}");
    assert_eq!(pressed(&body, "Rate 4 out of 5"), Some(false), "{body}");
}

#[tokio::test]
async fn stars_outside_one_to_five_and_not_a_number_are_refused() {
    let harness = Harness::start("ratings-invalid").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &owner, "Pancakes").await;
    let path = format!("/recipes/{id}/rating");

    for stars in ["stars=0", "stars=6", "stars=x", "stars=", "other=3"] {
        assert_eq!(
            post(&harness, &owner, &path, stars).await,
            StatusCode::BAD_REQUEST,
            "{stars}"
        );
    }
    let body = page(&harness, &owner, &format!("/recipes/{id}")).await;
    assert!(body.contains("No ratings yet"), "{body}");
}

#[tokio::test]
async fn an_unknown_recipe_is_not_found_for_ratings_and_favourites() {
    let harness = Harness::start("ratings-unknown").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;

    assert_eq!(
        post(&harness, &owner, "/recipes/999/rating", "stars=3").await,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        post(&harness, &owner, "/recipes/999/favourite", "on=true").await,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn signed_out_requests_are_sent_to_sign_in() {
    let harness = Harness::start("ratings-signed-out").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &owner, "Pancakes").await;

    for path in [
        format!("/recipes/{id}/rating"),
        format!("/recipes/{id}/favourite"),
    ] {
        let response = harness.post_form(&path, None, "stars=3&on=true").await;
        assert_eq!(response.status(), StatusCode::SEE_OTHER, "{path}");
        assert_eq!(
            header(&response, LOCATION).as_deref(),
            Some("/auth/login"),
            "{path}"
        );
    }
}

#[tokio::test]
async fn favourites_are_per_user_and_toggle() {
    let harness = Harness::start("favourites-toggle").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let other = sign_in_as(&harness, OTHER, &[]).await;
    let id = create(&harness, &owner, "Pancakes").await;
    let path = format!("/recipes/{id}");
    let add = "Add to favourites";
    let remove = "Remove from favourites";

    let body = page(&harness, &owner, &path).await;
    assert!(body.contains(add), "{body}");
    assert!(!body.contains(remove), "{body}");

    let response = harness
        .post_form(&format!("{path}/favourite"), Some(&owner), "on=true")
        .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(header(&response, LOCATION).as_deref(), Some(path.as_str()));

    let body = page(&harness, &owner, &path).await;
    assert!(body.contains(remove), "{body}");
    let body = page(&harness, &other, &path).await;
    assert!(body.contains(add), "{body}");
    assert!(!body.contains(remove), "{body}");

    post(&harness, &owner, &format!("{path}/favourite"), "on=false").await;
    let body = page(&harness, &owner, &path).await;
    assert!(body.contains(add), "{body}");
}

#[tokio::test]
async fn a_favourite_value_other_than_true_or_false_is_refused() {
    let harness = Harness::start("favourites-invalid").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let id = create(&harness, &owner, "Pancakes").await;
    assert_eq!(
        post(
            &harness,
            &owner,
            &format!("/recipes/{id}/favourite"),
            "on=yes"
        )
        .await,
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn the_favourites_filter_lists_only_the_viewers_favourites() {
    let harness = Harness::start("favourites-filter").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    let other = sign_in_as(&harness, OTHER, &[]).await;
    let pancakes = create(&harness, &owner, "Pancakes").await;
    let roast = create(&harness, &owner, "Roast").await;
    post(
        &harness,
        &owner,
        &format!("/recipes/{pancakes}/favourite"),
        "on=true",
    )
    .await;
    post(
        &harness,
        &other,
        &format!("/recipes/{roast}/favourite"),
        "on=true",
    )
    .await;

    let body = page(&harness, &owner, "/recipes?favourites=1").await;
    assert!(body.contains("Pancakes</a>"), "{body}");
    assert!(!body.contains("Roast</a>"), "{body}");
    assert!(body.contains("♥ Favourite"), "{body}");
    assert!(
        body.contains("<a href=\"/recipes\" aria-current=\"page\">Favourites</a>"),
        "{body}"
    );
    assert!(body.contains("<a href=\"/recipes\">All</a>"), "{body}");

    let body = page(&harness, &other, "/recipes?favourites=1").await;
    assert!(body.contains("Roast</a>"), "{body}");
    assert!(!body.contains("Pancakes</a>"), "{body}");

    let body = page(&harness, &owner, "/recipes").await;
    assert!(body.contains("Pancakes</a>"), "{body}");
    assert!(body.contains("Roast</a>"), "{body}");
    assert!(
        body.contains("<a href=\"/recipes?favourites=1\">Favourites</a>"),
        "{body}"
    );
    assert!(
        body.contains("<a href=\"/recipes\" aria-current=\"page\">All</a>"),
        "{body}"
    );
}

#[tokio::test]
async fn the_favourites_filter_with_no_favourites_says_so() {
    let harness = Harness::start("favourites-empty").await;
    let owner = sign_in_as(&harness, OWNER, &[]).await;
    create(&harness, &owner, "Pancakes").await;
    let body = page(&harness, &owner, "/recipes?favourites=1").await;
    assert!(body.contains("You have no favourites yet."), "{body}");
}
