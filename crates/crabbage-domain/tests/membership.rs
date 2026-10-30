//! Roles and households derived from login groups.

use crabbage_domain::membership::{Membership, Role, has_household_features};

const ADMIN: &str = "crabbage-admin";
const PREFIX: &str = "crabbage-household-";

fn groups(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_owned()).collect()
}

#[test]
fn admin_group_makes_an_admin() {
    let membership = Membership::from_groups(&groups(&[ADMIN]), ADMIN, PREFIX);
    assert_eq!(membership.role, Role::Admin);
}

#[test]
fn other_groups_make_a_member() {
    let membership = Membership::from_groups(&groups(&["crabbage-household-smith"]), ADMIN, PREFIX);
    assert_eq!(membership.role, Role::Member);
}

#[test]
fn no_groups_make_a_member_without_households() {
    let membership = Membership::from_groups(&[], ADMIN, PREFIX);
    assert_eq!(membership.role, Role::Member);
    assert_eq!(membership.households, Vec::<String>::new());
}

#[test]
fn only_groups_with_a_name_after_the_prefix_are_households() {
    let membership = Membership::from_groups(
        &groups(&[
            "crabbage-household-smith",
            "crabbage-household-",
            "crabbage-householdx",
            "other-crabbage-household-jones",
            "crabbage-admin-x",
        ]),
        ADMIN,
        PREFIX,
    );
    assert_eq!(membership.households, ["crabbage-household-smith"]);
}

#[test]
fn households_are_deduplicated_and_sorted() {
    let membership = Membership::from_groups(
        &groups(&[
            "crabbage-household-b",
            "crabbage-household-a",
            "crabbage-household-b",
        ]),
        ADMIN,
        PREFIX,
    );
    assert_eq!(
        membership.households,
        ["crabbage-household-a", "crabbage-household-b"]
    );
}

#[test]
fn household_features_need_an_active_household() {
    assert!(!has_household_features(None));
    assert!(has_household_features(Some(1)));
}
