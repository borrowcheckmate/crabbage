//! Writes the rendered pages to `CRABBAGE_SCREENSHOT_DIR` so they can be photographed for review.
//! Ignored by default: run with `cargo test --features crabbage/ssr --test server -- --ignored`.

use std::path::PathBuf;

use crabbage_store::catalog::{ingredients, sections};

use crate::common::{Harness, body_text, session_token};

#[tokio::test]
#[ignore = "writes review screenshots' HTML; needs CRABBAGE_SCREENSHOT_DIR"]
async fn write_pages() {
    let dir = PathBuf::from(std::env::var("CRABBAGE_SCREENSHOT_DIR").expect("dir is set"));
    std::fs::create_dir_all(&dir).expect("dir is created");
    let harness = Harness::start("screenshots").await;
    let produce = sections::create(&harness.pool, "Produce")
        .await
        .expect("section");
    sections::create(&harness.pool, "Dairy")
        .await
        .expect("section");
    let tomato = ingredients::create(&harness.pool, "Tomato")
        .await
        .expect("ingredient");
    ingredients::create(&harness.pool, "Onion")
        .await
        .expect("ingredient");
    ingredients::set_section(&harness.pool, tomato, Some(produce))
        .await
        .expect("section set");
    ingredients::add_alias(&harness.pool, tomato, "Pomodoro")
        .await
        .expect("alias");
    sqlx::query("INSERT INTO ingredients (name, pending_review) VALUES ('Okra', 1)")
        .execute(&harness.pool)
        .await
        .expect("pending ingredient");

    let pages = [
        ("admin", &["crabbage-admin"][..], "home", "/"),
        (
            "admin",
            &["crabbage-admin"][..],
            "ingredients",
            "/admin/ingredients",
        ),
        (
            "admin",
            &["crabbage-admin"][..],
            "sections",
            "/admin/sections",
        ),
        ("admin", &["crabbage-admin"][..], "review", "/admin/review"),
        ("member", &[][..], "home", "/"),
        ("member", &[][..], "ingredients", "/admin/ingredients"),
    ];
    for (who, groups, name, path) in pages {
        harness.issuer.set_groups(groups);
        let token = session_token(&harness.sign_in().await).expect("sign-in sets a session");
        let body = body_text(harness.get(path, Some(&token)).await).await;
        std::fs::write(dir.join(format!("{who}-{name}.html")), body).expect("page is written");
    }
}
