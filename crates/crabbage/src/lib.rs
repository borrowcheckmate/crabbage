//! The crabbage web app: a Leptos shell rendered on the server and hydrated in the browser.

#[expect(
    unreachable_pub,
    clippy::same_name_method,
    reason = "the #[component] props builder adds pub methods to a private type, and a builder method named like the trait method"
)]
mod app;

pub use app::{App, shell};

#[cfg(feature = "ssr")]
mod access;

#[cfg(feature = "ssr")]
mod api;

#[expect(
    unreachable_pub,
    clippy::same_name_method,
    reason = "the #[component] props builder adds pub methods to a private type, and a builder method named like the trait method"
)]
mod admin;

mod catalog;

mod notice;

#[expect(
    unreachable_pub,
    clippy::same_name_method,
    reason = "the #[component] props builder adds pub methods to a private type, and a builder method named like the trait method"
)]
mod recipes;

mod user;

#[cfg(feature = "ssr")]
mod config;

mod i18n;

#[cfg(feature = "ssr")]
pub use config::{AiConfig, Config, ConfigError};

pub use i18n::Language;

#[cfg(feature = "ssr")]
mod auth;

#[cfg(feature = "ssr")]
pub use auth::SessionUser;

#[cfg(feature = "ssr")]
mod household;

#[cfg(feature = "ssr")]
mod server;

#[cfg(feature = "ssr")]
pub use server::router;

#[cfg(feature = "ssr")]
mod state;

#[cfg(feature = "hydrate")]
use i18n::document_language;
#[cfg(feature = "hydrate")]
use leptos::prelude::provide_context;
#[cfg(feature = "hydrate")]
use wasm_bindgen::prelude::wasm_bindgen;

/// Hydrates the server-rendered page in the browser, in the language the server rendered it in.
#[cfg(feature = "hydrate")]
#[wasm_bindgen]
pub fn hydrate() {
    let language = document_language();
    leptos::mount::hydrate_body(move || {
        provide_context(language);
        App()
    });
}
