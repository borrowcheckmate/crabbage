use std::convert::Infallible;

use axum::Extension;
use axum::Router;
use axum::extract::{Form, Path, State};
use axum::http::StatusCode;
use axum::response::Redirect;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing::{get, post};
use crabbage_store::households::active_household;
use crabbage_store::shopping::{self, ShoppingItem};
use futures_util::stream;
use serde::Deserialize;
use tokio::sync::broadcast;

use crate::SessionUser;
use crate::access::internal_error;
use crate::state::AppState;

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/shopping/events", get(events))
        .route("/shopping/items", post(add_item))
        .route("/shopping/checked", post(check_items))
        .route("/shopping/items/{id}/delete", post(delete_item))
        .route("/shopping/clear-checked", post(clear_checked))
}

#[derive(Deserialize)]
struct TextForm {
    #[serde(default)]
    text: String,
}

#[derive(Deserialize)]
struct CheckedItemsForm {
    ids: String,
    on: String,
}

async fn household(state: &AppState, user: &SessionUser) -> Result<i64, StatusCode> {
    active_household(&state.pool, user.0)
        .await
        .map_err(internal_error)?
        .ok_or(StatusCode::NOT_FOUND)
}

async fn events(
    State(state): State<AppState>,
    Extension(user): Extension<SessionUser>,
) -> Result<Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>>, StatusCode> {
    let household = household(&state, &user).await?;
    let receiver = state.shopping.subscribe(household);
    let items = shopping::list(&state.pool, household)
        .await
        .map_err(internal_error)?;
    let initial = list_event(&items)?;
    let pool = state.pool.clone();
    let stream = stream::unfold(
        (receiver, pool, household, Some(initial)),
        |(mut receiver, pool, household, initial)| async move {
            if let Some(event) = initial {
                return Some((Ok(event), (receiver, pool, household, None)));
            }
            match receiver.recv().await {
                Ok(()) | Err(broadcast::error::RecvError::Lagged(_)) => {
                    let Ok(items) = shopping::list(&pool, household).await else {
                        return None;
                    };
                    let Ok(event) = list_event(&items) else {
                        return None;
                    };
                    Some((Ok(event), (receiver, pool, household, None)))
                }
                Err(broadcast::error::RecvError::Closed) => None,
            }
        },
    );
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

fn list_event(items: &[ShoppingItem]) -> Result<Event, StatusCode> {
    let json = serde_json::to_string(&items).map_err(internal_error)?;
    Ok(Event::default().event("list").data(json))
}

async fn add_item(
    State(state): State<AppState>,
    Extension(user): Extension<SessionUser>,
    Form(form): Form<TextForm>,
) -> Result<Redirect, StatusCode> {
    let text = form.text.trim();
    if text.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let household = household(&state, &user).await?;
    shopping::add_text(&state.pool, household, text)
        .await
        .map_err(internal_error)?;
    state.shopping.publish(household);
    Ok(Redirect::to("/shopping"))
}

async fn check_items(
    State(state): State<AppState>,
    Extension(user): Extension<SessionUser>,
    Form(form): Form<CheckedItemsForm>,
) -> Result<Redirect, StatusCode> {
    let checked = checked_value(&form.on)?;
    let ids = form
        .ids
        .split(',')
        .map(|id| id.parse::<i64>().map_err(|_error| StatusCode::BAD_REQUEST))
        .collect::<Result<Vec<_>, _>>()?;
    if ids.is_empty() || ids.iter().any(|id| *id <= 0) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let household = household(&state, &user).await?;
    let mut unique = ids.clone();
    unique.sort_unstable();
    unique.dedup();
    let changed = shopping::set_checked(&state.pool, household, &unique, checked)
        .await
        .map_err(internal_error)?;
    if changed != u64::try_from(unique.len()).map_err(internal_error)? {
        return Err(StatusCode::NOT_FOUND);
    }
    state.shopping.publish(household);
    Ok(Redirect::to("/shopping"))
}

fn checked_value(value: &str) -> Result<bool, StatusCode> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

async fn delete_item(
    State(state): State<AppState>,
    Extension(user): Extension<SessionUser>,
    Path(id): Path<i64>,
) -> Result<Redirect, StatusCode> {
    let household = household(&state, &user).await?;
    if !shopping::remove(&state.pool, household, id)
        .await
        .map_err(internal_error)?
    {
        return Err(StatusCode::NOT_FOUND);
    }
    state.shopping.publish(household);
    Ok(Redirect::to("/shopping"))
}

async fn clear_checked(
    State(state): State<AppState>,
    Extension(user): Extension<SessionUser>,
) -> Result<Redirect, StatusCode> {
    let household = household(&state, &user).await?;
    if shopping::clear_checked(&state.pool, household)
        .await
        .map_err(internal_error)?
        > 0
    {
        state.shopping.publish(household);
    }
    Ok(Redirect::to("/shopping"))
}
