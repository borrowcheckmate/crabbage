//! What each role may do in the app.

use crate::membership::Role;

/// Whether `role` may manage the catalog: ingredients, store sections and, later, categories.
#[must_use]
pub const fn can_manage_catalog(role: Role) -> bool {
    matches!(role, Role::Admin)
}
