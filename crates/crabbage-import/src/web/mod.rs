//! Recipe import from web pages: the schema.org `Recipe` in a page and the fetch that reads it.

mod address;
mod draft;
mod duration;
mod error;
mod fetch;
mod import;
mod jsonld;
mod language;

pub use address::is_public;
pub use draft::Draft;
pub use error::ImportError;
pub use fetch::{Fetched, Limits, fetch, fetch_bytes};
pub use import::import;
pub use jsonld::extract;
