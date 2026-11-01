//! Mealie recipe persistence.

#[expect(
    clippy::expect_used,
    reason = "test setup failures should stop the test"
)]
mod common;

use common::database;
use crabbage_store::catalog::ingredients::{create, create_pending_with_aliases, list};
use crabbage_store::recipes::{
    LineInput, RecipeInput, StepInput, create_mealie, imported_mealie_ids,
};
use crabbage_store::users::upsert_user;

#[tokio::test]
async fn mealie_recipe_ids_and_pending_ingredient_aliases_are_persisted() {
    let (_dir, pool) = database("mealie-import").await;
    let user = upsert_user(&pool, "subject", "Cook").await.unwrap();
    create(&pool, "Onion").await.unwrap();
    let ingredient = create_pending_with_aliases(
        &pool,
        "okra",
        &["Lady finger".to_owned(), "Onion".to_owned()],
    )
    .await
    .unwrap();
    let items = list(&pool).await.unwrap();
    let imported = items.iter().find(|item| item.id == ingredient).unwrap();
    assert!(imported.pending_review);
    assert_eq!(imported.aliases, ["Lady finger"]);
    let input = RecipeInput {
        title: "Soup".to_owned(),
        servings: 2,
        lines: vec![LineInput {
            amount: None,
            unit: None,
            ingredient: Some("Okra".to_owned()),
            note: None,
        }],
        steps: vec![StepInput {
            text: "Cook".to_owned(),
            duration_seconds: None,
            line_positions: vec![0],
        }],
        category_ids: Vec::new(),
    };
    create_mealie(&pool, user, &input, 1, "m-1").await.unwrap();
    assert_eq!(imported_mealie_ids(&pool).await.unwrap().len(), 1);
}
