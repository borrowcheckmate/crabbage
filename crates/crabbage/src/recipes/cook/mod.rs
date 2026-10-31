//! The cook page: one step at a time, with the steps marked done kept in the address.

mod links;
mod page;
mod query;
#[cfg(any(feature = "hydrate", test))]
mod swipe;

pub(crate) use links::start_href;
pub(crate) use page::CookPage;
