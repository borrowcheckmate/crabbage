//! Writes from many connections at once wait for each other instead of failing with a locked
//! database.

#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
mod common;

use common::database;
use crabbage_store::catalog::ingredients::{create, list, rename};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_creates_and_renames_all_succeed() {
    let (_dir, pool) = database("concurrent-writes").await;
    let tasks: Vec<_> = (0..40)
        .map(|index| {
            let pool = pool.clone();
            tokio::spawn(async move {
                let id = create(&pool, &format!("Ingredient {index}")).await?;
                rename(&pool, id, &format!("Renamed {index}")).await
            })
        })
        .collect();
    for task in tasks {
        assert!(task.await.unwrap().unwrap());
    }
    assert_eq!(list(&pool).await.unwrap().len(), 40);
}
