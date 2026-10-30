//! SQLite persistence for recipes and their related data.

mod error;
mod pool;

pub use error::StoreError;
pub use pool::open;
