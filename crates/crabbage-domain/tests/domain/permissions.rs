//! Catalog management is for admins only. Recipes: editing is open to the owner, the listed
//! editors and admins; deleting and managing editors to the owner and admins.

use crabbage_domain::membership::Role;
use crabbage_domain::permissions::{
    can_delete_recipe, can_edit_recipe, can_manage_catalog, can_manage_editors,
};

const OWNER: i64 = 1;
const EDITOR: i64 = 2;
const ADMIN: i64 = 3;
const OTHER: i64 = 4;
const EDITORS: [i64; 1] = [EDITOR];

#[test]
fn admins_manage_the_catalog() {
    assert!(can_manage_catalog(Role::Admin));
}

#[test]
fn members_do_not_manage_the_catalog() {
    assert!(!can_manage_catalog(Role::Member));
}

#[test]
fn the_owner_may_edit_delete_and_manage_editors() {
    assert!(can_edit_recipe(Role::Member, OWNER, OWNER, &EDITORS));
    assert!(can_delete_recipe(Role::Member, OWNER, OWNER));
    assert!(can_manage_editors(Role::Member, OWNER, OWNER));
}

#[test]
fn a_listed_editor_may_only_edit() {
    assert!(can_edit_recipe(Role::Member, EDITOR, OWNER, &EDITORS));
    assert!(!can_delete_recipe(Role::Member, EDITOR, OWNER));
    assert!(!can_manage_editors(Role::Member, EDITOR, OWNER));
}

#[test]
fn an_admin_may_do_everything_to_any_recipe() {
    assert!(can_edit_recipe(Role::Admin, ADMIN, OWNER, &EDITORS));
    assert!(can_delete_recipe(Role::Admin, ADMIN, OWNER));
    assert!(can_manage_editors(Role::Admin, ADMIN, OWNER));
}

#[test]
fn another_member_may_do_nothing_to_someone_elses_recipe() {
    assert!(!can_edit_recipe(Role::Member, OTHER, OWNER, &EDITORS));
    assert!(!can_delete_recipe(Role::Member, OTHER, OWNER));
    assert!(!can_manage_editors(Role::Member, OTHER, OWNER));
}

#[test]
fn an_editor_removed_from_the_list_loses_editing() {
    assert!(!can_edit_recipe(Role::Member, EDITOR, OWNER, &[]));
}
