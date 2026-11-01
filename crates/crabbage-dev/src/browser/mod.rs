//! Hydrated browser checks for the local demo application.

mod assets;
mod interactions;
mod navigation;
mod no_javascript;
mod run;

#[cfg(test)]
mod tests;

pub use run::run;
