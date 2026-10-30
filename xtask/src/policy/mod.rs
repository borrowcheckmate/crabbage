//! Repository rules and the checks that enforce them.

mod comments;
mod commit;
mod identity;
mod layout;
mod leaks;
mod length;
mod module_root;
mod repository;
mod violation;

pub(crate) use commit::{check_commit_message, check_pull_request};
pub(crate) use identity::{check_identities, email_from_ident};
pub(crate) use repository::check_repository;
pub(crate) use violation::Violation;
