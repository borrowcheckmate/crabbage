//! The Mealie migration admin page and its background run.

mod page;
#[cfg(feature = "ssr")]
mod runner;

pub(crate) use page::MealiePage;
#[cfg(feature = "ssr")]
pub(crate) use page::RunState;
#[cfg(feature = "ssr")]
pub(crate) use runner::routes;
