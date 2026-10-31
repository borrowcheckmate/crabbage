//! SQLite persistence for recipes and their related data.

pub mod catalog;
mod error;
pub mod households;
pub mod login_attempts;
pub mod planner;
mod pool;
pub mod recipes;
pub mod sessions;
pub mod shopping;
pub mod users;

pub use error::StoreError;
use pool::begin_write;
pub use pool::open;
