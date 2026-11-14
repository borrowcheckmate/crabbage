//! The review queue: ingredients that users added and an admin still has to approve.

use leptos::form::ActionForm;
use leptos::prelude::*;

use crate::catalog::SectionRow;
use crate::catalog::ingredients::{ApproveIngredient, list_pending_ingredients};
use crate::catalog::sections::list_sections;
use crate::i18n::t;
use crate::notice::{error_notice, failure, message};
use crate::user::current_user;

type ReviewPageData = (
    Result<crate::user::SignedInUser, ServerFnError>,
    Option<Result<(Vec<crate::catalog::IngredientRow>, Vec<SectionRow>), ServerFnError>>,
);

/// The review page. Admins see the pending ingredients, everyone else a message. The data is loaded
/// inside one suspense.
#[component]
pub(crate) fn ReviewPage() -> impl IntoView {
    let approve = ServerAction::<ApproveIngredient>::new();
    let data = Resource::new(move || approve.version().get(), |_| load_review());
    view! {
        <Suspense fallback=|| view! { <p class="state-notice" role="status">{t("admin.loading")}</p> }>
            {move || {
                Suspend::new(async move { review_page(data.await, approve) })
            }}
        </Suspense>
    }
}

async fn load_review() -> ReviewPageData {
    let user = current_user().await;
    let pending = if user.as_ref().is_ok_and(|user| user.is_admin) {
        Some(pending_with_sections().await)
    } else {
        None
    };
    (user, pending)
}

async fn pending_with_sections()
-> Result<(Vec<crate::catalog::IngredientRow>, Vec<SectionRow>), ServerFnError> {
    Ok((list_pending_ingredients().await?, list_sections().await?))
}

fn section_name(sections: &[SectionRow], id: Option<i64>) -> String {
    id.and_then(|id| sections.iter().find(|section| section.id == id))
        .map_or_else(
            || t("admin.label.no_section").to_owned(),
            |section| section.name.clone(),
        )
}

fn review_page(
    (user, pending): ReviewPageData,
    approve: ServerAction<ApproveIngredient>,
) -> AnyView {
    match user {
        Ok(user) if user.is_admin => {
            let pending = pending.unwrap_or_else(|| Ok((Vec::new(), Vec::new())));
            view! {
                <section class="admin-page"><h1>{t("admin.review.heading")}</h1>
                {move || approve.value().with(|value| failure(value.as_ref()))}
                {match pending {
                    Ok((rows, _)) if rows.is_empty() => {
                        view! { <p>{t("admin.review.empty")}</p> }.into_any()
                    }
                    Ok((rows, sections)) => view! {
                        <div class="admin-list">
                        {rows
                        .into_iter()
                        .map(|row| {
                            let id = row.id;
                            let section = section_name(&sections, row.section_id);
                            view! {
                                <div class="admin-row">
                                    <ActionForm action=approve>
                                        <p><a href=format!("/admin/ingredients/{id}")>{row.name}</a>" · "{section}</p>
                                        <input type="hidden" name="id" value=id/>
                                        <button>{t("admin.button.approve")}</button>
                                    </ActionForm>
                                </div>
                            }
                        })
                        .collect_view()}
                        </div>
                    }.into_any(),
                    Err(error) => error_notice(&message(&error)),
                }}
                </section>
            }
            .into_any()
        }
        Ok(_) | Err(_) => error_notice(t("admin.not_allowed")),
    }
}
