//! Catalog management is for admins only.

use crabbage_domain::membership::Role;
use crabbage_domain::permissions::can_manage_catalog;

#[test]
fn admins_manage_the_catalog() {
    assert!(can_manage_catalog(Role::Admin));
}

#[test]
fn members_do_not_manage_the_catalog() {
    assert!(!can_manage_catalog(Role::Member));
}
