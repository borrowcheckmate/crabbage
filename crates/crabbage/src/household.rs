//! Switching the active household of the signed-in user.

use axum::Form;
use axum::Router;
use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::Redirect;
use axum::routing::post;
use crabbage_store::households;
use serde::Deserialize;

use crate::SessionUser;
use crate::state::AppState;

#[derive(Deserialize)]
struct SwitchForm {
    household_id: i64,
}

pub(crate) fn routes() -> Router<AppState> {
    Router::new().route("/household", post(switch))
}

async fn switch(
    State(state): State<AppState>,
    Extension(user): Extension<SessionUser>,
    Form(form): Form<SwitchForm>,
) -> Result<Redirect, StatusCode> {
    match households::set_active_household(&state.pool, user.0, form.household_id).await {
        Ok(true) => Ok(Redirect::to("/")),
        Ok(false) => Err(StatusCode::FORBIDDEN),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub(crate) fn household_label(name: Option<&str>, group_name: &str, prefix: &str) -> String {
    if let Some(name) = name.map(str::trim).filter(|name| !name.is_empty()) {
        return name.to_owned();
    }
    group_name
        .strip_prefix(prefix)
        .filter(|suffix| !suffix.is_empty())
        .unwrap_or(group_name)
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::household_label;

    #[test]
    fn uses_a_non_empty_custom_name() {
        assert_eq!(household_label(Some(" Maple "), "g", "p"), "Maple");
    }

    #[test]
    fn strips_the_prefix_when_no_custom_name_exists() {
        assert_eq!(household_label(None, "prefix-maple", "prefix-"), "maple");
        assert_eq!(
            household_label(Some(" "), "prefix-maple", "prefix-"),
            "maple"
        );
    }

    #[test]
    fn keeps_the_group_when_prefix_does_not_match_or_strips_to_empty() {
        assert_eq!(household_label(None, "maple", "prefix-"), "maple");
        assert_eq!(household_label(None, "prefix-", "prefix-"), "prefix-");
    }
}
