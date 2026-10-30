//! The review queue: ingredients that users added and an admin still has to approve.

use leptos::form::ActionForm;
use leptos::prelude::*;

use super::notice::{error_notice, failure, message};
use crate::catalog::ingredients::{ApproveIngredient, list_pending_ingredients};
use crate::i18n::t;
use crate::user::current_user;

/// The review page. Admins see the pending ingredients, everyone else a message. The data is loaded
/// inside one suspense.
#[component]
pub(crate) fn ReviewPage() -> impl IntoView {
    let approve = ServerAction::<ApproveIngredient>::new();
    view! {
        <Suspense fallback=|| ()>
            {move || {
                let _version = approve.version().get();
                Suspend::new(async move { review_page(approve).await })
            }}
        </Suspense>
    }
}

async fn review_page(approve: ServerAction<ApproveIngredient>) -> AnyView {
    match current_user().await {
        Ok(user) if user.is_admin => {
            let pending = list_pending_ingredients().await;
            view! {
                <h1>{t("admin.review.heading")}</h1>
                {move || approve.value().with(|value| failure(value.as_ref()))}
                {match pending {
                    Ok(rows) if rows.is_empty() => {
                        view! { <p>{t("admin.review.empty")}</p> }.into_any()
                    }
                    Ok(rows) => rows
                        .into_iter()
                        .map(|row| {
                            let id = row.id;
                            view! {
                                <div class="card">
                                    <ActionForm action=approve>
                                        <p>{row.name}</p>
                                        <input type="hidden" name="id" value=id/>
                                        <button>{t("admin.button.approve")}</button>
                                    </ActionForm>
                                </div>
                            }
                        })
                        .collect_view()
                        .into_any(),
                    Err(error) => error_notice(&message(&error)),
                }}
            }
            .into_any()
        }
        Ok(_) | Err(_) => error_notice(t("admin.not_allowed")),
    }
}
