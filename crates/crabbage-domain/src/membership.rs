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

impl Membership {
    /// Derives the membership from the `groups` of a login.
    ///
    /// The user is an admin when `groups` contains `admin_group`. Every group that starts with
    /// `household_prefix` and has a name after it is a household, named by the whole group.
    #[must_use]
    pub fn from_groups(groups: &[String], admin_group: &str, household_prefix: &str) -> Self {
        let role = if groups.iter().any(|group| group == admin_group) {
            Role::Admin
        } else {
            Role::Member
        };
        let mut households: Vec<String> = groups
            .iter()
            .filter(|group| {
                group
                    .strip_prefix(household_prefix)
                    .is_some_and(|name| !name.is_empty())
            })
            .cloned()
            .collect();
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
