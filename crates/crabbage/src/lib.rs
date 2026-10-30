//! The crabbage web app: a Leptos shell rendered on the server and hydrated in the browser.

#[expect(
    unreachable_pub,
    clippy::same_name_method,
    reason = "the #[component] props builder adds pub methods to a private type, and a builder method named like the trait method"
)]
mod app;

pub use app::{App, shell};

#[cfg(feature = "ssr")]
mod server;

#[cfg(feature = "ssr")]
pub use server::router;

#[cfg(feature = "hydrate")]
use wasm_bindgen::prelude::wasm_bindgen;

/// Hydrates the server-rendered page in the browser.
#[cfg(feature = "hydrate")]
#[wasm_bindgen]
pub fn hydrate() {
    leptos::mount::hydrate_body(App);
}
