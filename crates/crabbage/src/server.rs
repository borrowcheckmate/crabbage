use std::error::Error;
use std::sync::Arc;

use axum::Router;
use axum::middleware::from_fn_with_state;
use axum::routing::get;
use leptos::config::LeptosOptions;
use leptos::prelude::provide_context;
use leptos_axum::{LeptosRoutes, generate_route_list};
use sqlx::sqlite::SqlitePool;
use tokio::sync::OnceCell;

use crate::api;
use crate::app::{App, shell};
use crate::auth;
use crate::config::Config;
use crate::household;
use crate::images;
use crate::recipes::photo_routes;
use crate::state::AppState;

/// Builds the router for the app.
///
/// It serves the health check, sign-in, the rendered pages and the site files. Every route except
/// the health check, sign-in and the site files needs a session. The `OpenID` Connect provider is
/// not contacted here; sign-in discovers it on first use.
///
/// # Errors
///
/// Returns an error if the HTTP client for the provider cannot be built.
pub fn router(
    options: LeptosOptions,
    config: &Config,
    pool: SqlitePool,
) -> Result<Router, Box<dyn Error>> {
    let http = auth::http_client()?;
    let routes = generate_route_list(App);
    let state = AppState {
        options: options.clone(),
        pool,
        http,
        config: Arc::new(config.clone()),
        oidc: Arc::new(OnceCell::new()),
        admin_group: config.admin_group.clone(),
        household_group_prefix: config.household_group_prefix.clone(),
        language: config.language,
    };
    let language = state.language;
    let context_pool = state.pool.clone();
    let context_data_dir = config.data_dir.clone();
    Ok(Router::<AppState>::new()
        .route("/healthz", get(|| async { "ok" }))
        .merge(auth::routes())
        .merge(household::routes())
        .merge(api::routes())
        .merge(images::routes())
        .merge(photo_routes())
        .leptos_routes_with_context(
            &state,
            routes,
            move || {
                provide_context(context_pool.clone());
                provide_context(language);
                provide_context(context_data_dir.clone());
            },
            move || shell(options.clone()),
        )
        .fallback(leptos_axum::file_and_error_handler_with_context::<
            AppState,
            _,
        >(move || provide_context(language), shell))
        .layer(from_fn_with_state(state.clone(), auth::require_session))
        .with_state(state))
}
