//! What the signed-in user may do to one recipe. The owner, the listed editors and admins may
//! change it; the owner and admins may delete it and manage its editors.

use crabbage_domain::membership::Role;
use crabbage_domain::permissions::{can_delete_recipe, can_edit_recipe, can_manage_editors};
use crabbage_store::recipes::{Recipe, get};
use leptos::prelude::ServerFnError;
use sqlx::sqlite::SqlitePool;

use crate::access::{refuse, role, store_failure};
use crate::i18n::t;

/// The changes a signed-in user may ask for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Change {
    /// Change the title, servings, lines and steps.
    Edit,
    /// Delete the recipe.
    Delete,
    /// Add and remove editors.
    ManageEditors,
}

/// What one user may do to one recipe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Rights {
    /// Whether the user may change the recipe.
    pub(crate) edit: bool,
    /// Whether the user may delete the recipe.
    pub(crate) delete: bool,
    /// Whether the user may add and remove editors.
    pub(crate) manage_editors: bool,
}

impl Rights {
    /// The rights of user `user` with `role` to `recipe`.
    pub(crate) fn of(role: Role, user: i64, recipe: &Recipe) -> Self {
        let editors: Vec<i64> = recipe.editors.iter().map(|person| person.id).collect();
        Self {
            edit: can_edit_recipe(role, user, recipe.owner_id, &editors),
            delete: can_delete_recipe(role, user, recipe.owner_id),
            manage_editors: can_manage_editors(role, user, recipe.owner_id),
        }
    }
}

/// The recipe `id`, or the missing-item error when there is none.
pub(crate) async fn load(pool: &SqlitePool, id: i64) -> Result<Recipe, ServerFnError> {
    get(pool, id)
        .await
        .map_err(|error| store_failure(&error))?
        .ok_or_else(|| ServerFnError::new(t("catalog.error.missing")))
}

/// Checks that user `user` may make `change` to recipe `id`. Otherwise the response is 403.
///
/// # Errors
///
/// Returns an error for a missing recipe, a refused change and a failing store.
pub(crate) async fn permit(
    pool: &SqlitePool,
    user: i64,
    id: i64,
    change: Change,
) -> Result<(), ServerFnError> {
    let recipe = load(pool, id).await?;
    let rights = Rights::of(role(pool, user).await?, user, &recipe);
    let allowed = match change {
        Change::Edit => rights.edit,
        Change::Delete => rights.delete,
        Change::ManageEditors => rights.manage_editors,
    };
    if allowed {
        return Ok(());
    }
    Err(refuse(t("recipe.not_allowed")))
}
