//! Shared test harness: the app router on a temporary database, with a fake identity provider.

mod helpers;
mod issuer;

pub(crate) use helpers::*;
pub(crate) use issuer::FakeIssuer;
