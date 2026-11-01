use std::error::Error;
use std::sync::{Arc, OnceLock};

use axum::Router;
use axum::extract::Request;
use axum::http::{HeaderValue, header};
use axum::middleware::{Next, from_fn, from_fn_with_state};
use axum::response::Response;
use axum::routing::get;
use leptos::config::LeptosOptions;
use leptos::prelude::provide_context;
use leptos_axum::{AxumRouteListing, LeptosRoutes, generate_route_list};
use sqlx::sqlite::SqlitePool;
use tokio::sync::OnceCell;

use crate::admin::mealie::routes as mealie_routes;
use crate::api;
use crate::app::{App, shell};
use crate::auth;
use crate::config::Config;
use crate::household;
use crate::images;
use crate::planner::routes as planner_routes;
use crate::recipes::{PhotoDrafts, import_photo_routes, photo_routes, rating_routes};
use crate::shopping::routes as shopping_routes;
use crate::state::AppState;

/// The page routes, generated once per process.
///
/// Generating them suppresses resource loading process-wide, so a page rendered at the same time
/// waits forever (leptos-rs/leptos#4773). Every router shares the first list instead.
fn route_list() -> Vec<AxumRouteListing> {
    static ROUTES: OnceLock<Vec<AxumRouteListing>> = OnceLock::new();
    ROUTES.get_or_init(|| generate_route_list(App)).clone()
}

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
    let routes = route_list();
    let state = AppState {
        options: options.clone(),
        pool,
        http,
        config: Arc::new(config.clone()),
        oidc: Arc::new(OnceCell::new()),
        admin_group: config.admin_group.clone(),
        household_group_prefix: config.household_group_prefix.clone(),
        language: config.language,
        photo_drafts: Arc::new(PhotoDrafts::default()),
        shopping: crate::shopping::ShoppingHub::default(),
        mealie_migration: Arc::default(),
    };
    let language = state.language;
    let context_pool = state.pool.clone();
    let context_data_dir = config.data_dir.clone();
    let context_config = Arc::clone(&state.config);
    let context_drafts = Arc::clone(&state.photo_drafts);
    let context_migration = Arc::clone(&state.mealie_migration);
    Ok(Router::<AppState>::new()
        .route("/healthz", get(|| async { "ok" }))
        .merge(auth::routes())
        .merge(household::routes())
        .merge(api::routes())
        .merge(images::routes())
        .merge(photo_routes())
        .merge(import_photo_routes())
        .merge(rating_routes())
        .merge(shopping_routes())
        .merge(planner_routes())
        .merge(mealie_routes())
        .leptos_routes_with_context(
            &state,
            routes,
            move || {
                provide_context(context_pool.clone());
                provide_context(language);
                provide_context(context_data_dir.clone());
                provide_context(Arc::clone(&context_config));
                provide_context(Arc::clone(&context_drafts));
                provide_context(Arc::clone(&context_migration));
            },
            move || shell(options.clone()),
        )
        .fallback(leptos_axum::file_and_error_handler_with_context::<
            AppState,
            _,
        >(move || provide_context(language), shell))
        .layer(from_fn_with_state(state.clone(), auth::require_session))
        .layer(from_fn(revalidate_assets))
        .with_state(state))
}

/// Makes browsers revalidate the stylesheet and app bundle. Release builds give them hashed names,
/// but a build without the hash file serves them under fixed names.
async fn revalidate_assets(request: Request, next: Next) -> Response {
    let is_asset = request.uri().path().starts_with("/pkg/");
    let mut response = next.run(request).await;
    if is_asset {
        response
            .headers_mut()
            .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    }
    response
}
