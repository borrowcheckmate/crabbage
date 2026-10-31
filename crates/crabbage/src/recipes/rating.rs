//! Rating a recipe and keeping it as a favourite. Plain HTML forms post to these routes, which answer
//! with a redirect back to the recipe page. Anyone signed in may rate or favourite a recipe.

use axum::Extension;
use axum::Router;
use axum::extract::{Form, Path, State};
use axum::http::StatusCode;
use axum::response::Redirect;
use axum::routing::post;
use crabbage_store::recipes::{rate, set_favourite};
use serde::Deserialize;

use crate::SessionUser;
use crate::access::internal_error;
use crate::state::AppState;

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/recipes/{id}/rating", post(rate_recipe))
        .route("/recipes/{id}/favourite", post(favourite_recipe))
}

/// The stars field of the rating form, as typed. Checked by the route, so a bad value is a 400.
#[derive(Deserialize)]
struct RatingForm {
    #[serde(default)]
    stars: String,
}

/// The `on` field of the favourite form: `true` to add the recipe, `false` to remove it.
#[derive(Deserialize)]
struct FavouriteForm {
    #[serde(default)]
    on: String,
}

async fn rate_recipe(
    State(state): State<AppState>,
    Extension(user): Extension<SessionUser>,
    Path(id): Path<i64>,
    Form(form): Form<RatingForm>,
) -> Result<Redirect, StatusCode> {
    let stars = form
        .stars
        .parse::<i64>()
        .ok()
        .filter(|stars| (1..=5).contains(stars))
        .ok_or(StatusCode::BAD_REQUEST)?;
    let found = rate(&state.pool, id, user.0, stars)
        .await
        .map_err(internal_error)?;
    recipe_page(found, id)
}

async fn favourite_recipe(
    State(state): State<AppState>,
    Extension(user): Extension<SessionUser>,
    Path(id): Path<i64>,
    Form(form): Form<FavouriteForm>,
) -> Result<Redirect, StatusCode> {
    let favourite = match form.on.as_str() {
        "true" => true,
        "false" => false,
        _ => return Err(StatusCode::BAD_REQUEST),
    };
    let found = set_favourite(&state.pool, id, user.0, favourite)
        .await
        .map_err(internal_error)?;
    recipe_page(found, id)
}

/// The recipe page for `id` when the recipe was found, and 404 otherwise.
fn recipe_page(found: bool, id: i64) -> Result<Redirect, StatusCode> {
    if found {
        return Ok(Redirect::to(&format!("/recipes/{id}")));
    }
    Err(StatusCode::NOT_FOUND)
}
