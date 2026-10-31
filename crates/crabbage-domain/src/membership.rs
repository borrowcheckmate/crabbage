//! Roles and households, derived from the groups of an identity provider login.

/// What a user may do in the app.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// Administers the app.
    Admin,
    /// Uses the app.
    Member,
}

/// A user's role and the households they belong to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Membership {
    /// The user's role.
    pub role: Role,
    /// Full group names of the households: sorted and without duplicates.
    pub households: Vec<String>,
}

/// Claims used to derive a user's membership.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Claims {
    /// Full names of groups in the identity provider token.
    pub groups: Vec<String>,
    /// Optional role from the `crabbage_role` claim.
    pub role: Option<String>,
    /// Optional household values from the `crabbage_household` claim.
    pub households: Vec<String>,
}

impl Membership {
    /// Derives membership from the groups and custom claims of a login.
    ///
    /// The user is an admin when a group matches `admin_group` or the role claim is `admin`.
    /// Household groups retain their full names, while custom household claims supply names.
    #[must_use]
    pub fn from_claims(claims: &Claims, admin_group: &str, household_prefix: &str) -> Self {
        let role = if claims.groups.iter().any(|group| group == admin_group)
            || claims
                .role
                .as_deref()
                .is_some_and(|role| role.trim().eq_ignore_ascii_case("admin"))
        {
            Role::Admin
        } else {
            Role::Member
        };
        let mut households: Vec<String> = claims
            .groups
            .iter()
            .filter(|group| {
                group
                    .strip_prefix(household_prefix)
                    .is_some_and(|name| !name.is_empty())
            })
            .cloned()
            .collect();
        households.extend(
            claims
                .households
                .iter()
                .map(|household| household.trim())
                .filter(|household| !household.is_empty())
                .map(str::to_owned),
        );
        households.sort();
        households.dedup();
        Self { role, households }
    }
}

/// Whether the shopping list and the planner are available: they need an active household.
///
/// Recipes are available to every user.
#[must_use]
pub const fn has_household_features(active_household: Option<i64>) -> bool {
    active_household.is_some()
}
