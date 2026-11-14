//! Store integration tests, in one binary: each module covers one part of the store.

mod catalog_aliases;
mod catalog_categories;
mod catalog_ingredients;
mod catalog_merge;
mod catalog_sections;
#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod common;
mod concurrent_writes;
mod defaults;
mod households;
mod login_attempts;
mod open;
mod planner;
mod recipes_categories;
mod recipes_delete;
mod recipes_editors;
mod recipes_favourites;
mod recipes_images;
mod recipes_meal_types;
mod recipes_mealie;
mod recipes_ratings;
mod recipes_round_trip;
mod recipes_search;
mod recipes_update;
mod sessions;
mod settings;
mod shopping;
mod users;
