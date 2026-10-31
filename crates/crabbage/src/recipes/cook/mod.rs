//! The cook page: one step at a time, with the steps marked done kept in the address.

#[cfg(feature = "hydrate")]
mod alert;
mod links;
mod page;
mod query;
#[cfg(any(feature = "hydrate", test))]
mod swipe;
mod timers;
mod tray;
#[cfg(feature = "hydrate")]
mod tray_client;
#[cfg(any(feature = "hydrate", test))]
mod wake;
#[cfg(feature = "hydrate")]
mod wake_client;

pub(crate) use links::start_href;
pub(crate) use page::CookPage;
