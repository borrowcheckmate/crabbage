//! Recipe photos: the image types, the file names, the images directory, and serving the files.

mod files;
mod kind;
mod naming;
mod serve;

pub(crate) use files::{remove, write};
pub(crate) use kind::Kind;
pub(crate) use serve::routes;
#[cfg(feature = "ssr")]
mod crop;
#[cfg(feature = "ssr")]
pub(crate) use crop::crop;
