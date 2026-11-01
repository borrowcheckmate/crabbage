use axum::http::StatusCode;

use crate::common::{Harness, body_text, session_token};

#[tokio::test]
async fn admin_sees_households_and_can_rename_them_for_the_topbar() {
    let harness = Harness::start("admin-households-rename").await;
    harness
        .issuer
        .set_groups(&["admin", "crabbage-household-maple"]);
    let token = session_token(&harness.sign_in().await).expect("admin session");
    let page = body_text(harness.get("/admin/households", Some(&token)).await).await;
    assert!(page.contains("Households</h1>"), "{page}");
    assert!(page.contains("crabbage-household-maple"), "{page}");
    let home = body_text(harness.get("/", Some(&token)).await).await;
    assert!(home.contains(">maple</span>"), "{home}");
    let id: i64 = sqlx::query_scalar("SELECT id FROM households WHERE group_name = ?")
        .bind("crabbage-household-maple")
        .fetch_one(&harness.pool)
        .await
        .expect("household exists");
    let fields = url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs([("id", id.to_string()), ("name", "Maple Street".to_owned())])
        .finish();
    let response = harness
        .post_form("/api/rename_household", Some(&token), &fields)
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    let home = body_text(harness.get("/", Some(&token)).await).await;
    assert!(home.contains(">Maple Street</span>"), "{home}");
}

#[tokio::test]
async fn member_sees_admin_only_message_and_cannot_rename() {
    let harness = Harness::start("admin-households-member").await;
    harness.issuer.set_groups(&["crabbage-household-maple"]);
    let token = session_token(&harness.sign_in().await).expect("member session");
    let page = body_text(harness.get("/admin/households", Some(&token)).await).await;
    assert!(
        page.contains("You are not allowed to manage the catalog."),
        "{page}"
    );
    let response = harness
        .post_form("/api/rename_household", Some(&token), "id=1&name=Blocked")
        .await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
