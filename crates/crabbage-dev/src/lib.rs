//! Development-only identity provider shared by the app tests and demo.

mod demo;
#[expect(
    clippy::expect_used,
    clippy::missing_panics_doc,
    reason = "the original test fake treats key generation, signing, and poisoned test state as fatal"
)]
mod issuer;
mod recipes;

use recipes::DemoRecipe;

pub use demo::{enable_demo_ai, load_demo};
pub use issuer::FakeIssuer;
