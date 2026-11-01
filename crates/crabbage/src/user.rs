//! The signed-in user, for the pages that differ by role.

#![cfg_attr(
    not(feature = "ssr"),
    expect(
        clippy::unused_async_trait_impl,
        reason = "the client build replaces each server function body with a stub that does not await"
    )
)]

use leptos::prelude::ServerFnError;
use leptos::server;
use serde::{Deserialize, Serialize};

/// The signed-in user: their name and whether they may manage the catalog.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SignedInUser {
    /// The name from the latest login.
    pub(crate) name: String,
    /// Whether the user may manage the catalog.
    pub(crate) is_admin: bool,
    pub(crate) active_household: Option<String>,
    pub(crate) households: Vec<(i64, String)>,
}

#[server(prefix = "/api", endpoint = "current_user")]
pub(crate) async fn current_user() -> Result<SignedInUser, ServerFnError> {
    use crate::i18n::t;
    let (pool, user) = crate::access::signed_in().await?;
    let found = crabbage_store::users::find(&pool, user)
        .await
        .map_err(|error| crate::access::store_failure(&error))?
        .ok_or_else(|| ServerFnError::new(t("catalog.error.database")))?;
    let households = crabbage_store::households::households_of(&pool, user)
        .await
        .map_err(|error| crate::access::store_failure(&error))?;
    let active = crabbage_store::households::active_household(&pool, user)
        .await
        .map_err(|error| crate::access::store_failure(&error))?;
    let prefix = crate::access::config()?.household_group_prefix.clone();
    Ok(SignedInUser {
        name: found.name,
        is_admin: found.is_admin,
        active_household: households
            .iter()
            .find(|household| Some(household.id) == active)
            .map(|household| {
                crate::household::household_label(
                    household.name.as_deref(),
                    &household.group_name,
                    &prefix,
                )
            }),
        households: households
            .into_iter()
            .map(|household| {
                (
                    household.id,
                    crate::household::household_label(
                        household.name.as_deref(),
                        &household.group_name,
                        &prefix,
                    ),
                )
            })
            .collect(),
    })
}
