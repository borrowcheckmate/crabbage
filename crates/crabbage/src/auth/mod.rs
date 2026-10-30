//! Sign-in through the `OpenID` Connect provider and server-side sessions.

mod callback;
mod cookie;
mod error;
mod login;
mod logout;
mod middleware;
mod oidc;
mod routes;
mod session;

pub(crate) use error::AuthError;
pub(crate) use middleware::require_session;
pub(crate) use oidc::{Oidc, http_client};
pub use routes::SessionUser;
pub(crate) use routes::routes;
pub(crate) use session::now;
