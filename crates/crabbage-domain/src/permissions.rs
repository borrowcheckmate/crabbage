//! What each role may do in the app.

use crate::membership::Role;

/// Whether `role` may manage the catalog: ingredients, store sections and, later, categories.
#[must_use]
pub const fn can_manage_catalog(role: Role) -> bool {
    matches!(role, Role::Admin)
}

/// Whether `user_id` may change a recipe owned by `owner_id`: an admin, the owner or a listed
/// editor. Anyone signed in may create a recipe.
#[must_use]
pub fn can_edit_recipe(role: Role, user_id: i64, owner_id: i64, editors: &[i64]) -> bool {
    is_admin_or_owner(role, user_id, owner_id) || editors.contains(&user_id)
}

/// Whether `user_id` may delete a recipe owned by `owner_id`: an admin or the owner.
#[must_use]
pub const fn can_delete_recipe(role: Role, user_id: i64, owner_id: i64) -> bool {
    is_admin_or_owner(role, user_id, owner_id)
}

/// Whether `user_id` may add and remove the editors of a recipe owned by `owner_id`: an admin or
/// the owner.
#[must_use]
pub const fn can_manage_editors(role: Role, user_id: i64, owner_id: i64) -> bool {
    is_admin_or_owner(role, user_id, owner_id)
}

const fn is_admin_or_owner(role: Role, user_id: i64, owner_id: i64) -> bool {
    matches!(role, Role::Admin) || user_id == owner_id
}
