//! SQLite persistence for recipes and their related data.

pub mod catalog;
mod defaults;
mod error;
pub mod households;
pub mod login_attempts;
pub mod planner;
mod pool;
pub mod recipes;
pub mod sessions;
/// Persisted application settings.
pub mod settings;
pub mod shopping;
pub mod users;

pub use defaults::seed_defaults;
pub use error::StoreError;
use pool::begin_write;
pub use pool::open;
