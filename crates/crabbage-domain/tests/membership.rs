//! Roles and households derived from login groups.

use crabbage_domain::membership::{Claims, Membership, Role, has_household_features};

const ADMIN: &str = "admin";
const PREFIX: &str = "crabbage-household-";

fn groups(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_owned()).collect()
}

#[test]
fn admin_group_makes_an_admin() {
    let membership = Membership::from_claims(
        &Claims {
            groups: groups(&[ADMIN]),
            ..Claims::default()
        },
        ADMIN,
        PREFIX,
    );
    assert_eq!(membership.role, Role::Admin);
}

#[test]
fn other_groups_make_a_member() {
    let membership = Membership::from_claims(
        &Claims {
            groups: groups(&["crabbage-household-smith"]),
            ..Claims::default()
        },
        ADMIN,
        PREFIX,
    );
    assert_eq!(membership.role, Role::Member);
}

#[test]
fn no_groups_make_a_member_without_households() {
    let membership = Membership::from_claims(&Claims::default(), ADMIN, PREFIX);
    assert_eq!(membership.role, Role::Member);
    assert_eq!(membership.households, Vec::<String>::new());
}

#[test]
fn only_groups_with_a_name_after_the_prefix_are_households() {
    let membership = Membership::from_claims(
        &Claims {
            groups: groups(&[
                "crabbage-household-smith",
                "crabbage-household-",
                "crabbage-householdx",
                "other-crabbage-household-jones",
                "crabbage-admin-x",
            ]),
            ..Claims::default()
        },
        ADMIN,
        PREFIX,
    );
    assert_eq!(membership.households, ["crabbage-household-smith"]);
}

#[test]
fn households_are_deduplicated_and_sorted() {
    let membership = Membership::from_claims(
        &Claims {
            groups: groups(&[
                "crabbage-household-b",
                "crabbage-household-a",
                "crabbage-household-b",
            ]),
            ..Claims::default()
        },
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

#[test]
fn role_claim_admin_is_case_insensitive_and_trimmed() {
    let membership = Membership::from_claims(
        &Claims {
            role: Some(" Admin ".to_owned()),
            ..Claims::default()
        },
        ADMIN,
        PREFIX,
    );
    assert_eq!(membership.role, Role::Admin);
}

#[test]
fn other_role_claim_is_a_member() {
    let membership = Membership::from_claims(
        &Claims {
            role: Some("owner".to_owned()),
            ..Claims::default()
        },
        ADMIN,
        PREFIX,
    );
    assert_eq!(membership.role, Role::Member);
}

#[test]
fn household_claim_string_and_list_are_accepted() {
    let single = Membership::from_claims(
        &Claims {
            households: vec![" Vinkenhoeven ".to_owned()],
            ..Claims::default()
        },
        ADMIN,
        PREFIX,
    );
    assert_eq!(single.households, ["Vinkenhoeven"]);
    let many = Membership::from_claims(
        &Claims {
            households: groups(&["Zuid", "Noord"]),
            ..Claims::default()
        },
        ADMIN,
        PREFIX,
    );
    assert_eq!(many.households, ["Noord", "Zuid"]);
}

#[test]
fn claim_households_merge_with_groups_and_deduplicate() {
    let membership = Membership::from_claims(
        &Claims {
            groups: groups(&["crabbage-household-Vinkenhoeven", "crabbage-household-a"]),
            households: groups(&["Vinkenhoeven", " b "]),
            ..Claims::default()
        },
        ADMIN,
        PREFIX,
    );
    assert_eq!(
        membership.households,
        [
            "Vinkenhoeven",
            "b",
            "crabbage-household-Vinkenhoeven",
            "crabbage-household-a"
        ]
    );
}

#[test]
fn blank_claim_households_are_ignored() {
    let membership = Membership::from_claims(
        &Claims {
            households: groups(&["", "  "]),
            ..Claims::default()
        },
        ADMIN,
        PREFIX,
    );
    assert_eq!(membership.households, Vec::<String>::new());
}
