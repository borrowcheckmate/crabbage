use axum::Router;
use axum::routing::get;
use leptos::config::LeptosOptions;
use leptos_axum::{LeptosRoutes, generate_route_list};

use crate::app::{App, shell};

/// Builds the router for the app: the health check, the rendered pages and the site files.
pub fn router(options: LeptosOptions) -> Router {
    let routes = generate_route_list(App);
    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .leptos_routes(&options, routes, {
            let options = options.clone();
            move || shell(options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(options)
}
