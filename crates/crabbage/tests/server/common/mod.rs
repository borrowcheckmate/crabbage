//! Shared test harness: the app router on a temporary database, with a fake identity provider.

mod helpers;

pub(crate) use crabbage_dev::FakeIssuer;
pub(crate) use helpers::*;
