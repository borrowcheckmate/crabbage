//! SQLite persistence for recipes and their related data.

pub mod catalog;
mod error;
pub mod households;
pub mod login_attempts;
mod pool;
pub mod recipes;
pub mod sessions;
pub mod users;

pub use error::StoreError;
pub use pool::open;
