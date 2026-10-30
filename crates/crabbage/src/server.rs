use std::error::Error;
use std::sync::Arc;

use axum::Router;
use axum::middleware::from_fn_with_state;
use axum::routing::get;
use leptos::config::LeptosOptions;
use leptos::prelude::provide_context;
use leptos_axum::{LeptosRoutes, generate_route_list};
use sqlx::sqlite::SqlitePool;

use crate::app::{App, shell};
use crate::auth::{self, Oidc};
use crate::config::Config;
use crate::household;
use crate::state::AppState;

/// Builds the router for the app: the health check, sign-in, the rendered pages and the site
/// files. Every route except the health check, sign-in and the site files needs a session.
///
/// # Errors
///
/// Returns an error if the `OpenID` Connect provider cannot be discovered or the configured URLs
/// are invalid.
pub async fn router(
    options: LeptosOptions,
    config: &Config,
    pool: SqlitePool,
) -> Result<Router, Box<dyn Error>> {
    let http = auth::http_client()?;
    let oidc = Oidc::discover(config, &http).await?;
    let routes = generate_route_list(App);
    let state = AppState {
        options: options.clone(),
        pool,
        http,
        oidc: Arc::new(oidc),
        admin_group: config.admin_group.clone(),
        household_group_prefix: config.household_group_prefix.clone(),
        language: config.language,
    };
    let language = state.language;
    Ok(Router::<AppState>::new()
        .route("/healthz", get(|| async { "ok" }))
        .merge(auth::routes())
        .merge(household::routes())
        .leptos_routes_with_context(&state, routes, move || provide_context(language), {
            let options = options.clone();
            move || shell(options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler_with_context::<
            AppState,
            _,
        >(move || provide_context(language), shell))
        .layer(from_fn_with_state(state.clone(), auth::require_session))
        .with_state(state))
}
