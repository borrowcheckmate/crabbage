//! The recipe categories page: the add form, and a card per category with rename and delete.

use leptos::form::ActionForm;
use leptos::prelude::*;

use crate::catalog::CategoryRow;
use crate::catalog::categories::{CreateCategory, DeleteCategory, RenameCategory, list_categories};
use crate::i18n::t;
use crate::notice::{error_notice, failure, message};
use crate::user::current_user;

/// The categories page. Admins see the forms and the rows, everyone else a message. The data is
/// loaded inside one suspense.
#[component]
pub(crate) fn CategoriesPage() -> impl IntoView {
    let create = ServerAction::<CreateCategory>::new();
    let rename = ServerAction::<RenameCategory>::new();
    let delete = ServerAction::<DeleteCategory>::new();
    view! {
        <Suspense fallback=|| view! { <p class="state-notice" role="status">{t("admin.loading")}</p> }>
            {move || {
                let _versions = (create.version().get(), rename.version().get(), delete.version().get());
                Suspend::new(async move { category_page(create, rename, delete).await })
            }}
        </Suspense>
    }
}

async fn category_page(
    create: ServerAction<CreateCategory>,
    rename: ServerAction<RenameCategory>,
    delete: ServerAction<DeleteCategory>,
) -> AnyView {
    match current_user().await {
        Ok(user) if user.is_admin => {
            let rows = list_categories().await;
            view! {
                <section class="admin-page">
                <h1>{t("admin.categories.heading")}</h1>
                <section class="admin-card admin-create"><ActionForm action=create>
                    <label>{t("admin.label.name")} <input name="name" required/></label>
                    <button>{t("admin.button.add")}</button>
                </ActionForm></section>
                {move || create.value().with(|value| failure(value.as_ref()))}
                {move || rename.value().with(|value| failure(value.as_ref()))}
                {move || delete.value().with(|value| failure(value.as_ref()))}
                {category_rows(rows, rename, delete)}
                </section>
            }
            .into_any()
        }
        Ok(_) | Err(_) => error_notice(t("admin.not_allowed")),
    }
}

/// The cards of the category list, or the message that explains why they are missing.
fn category_rows(
    rows: Result<Vec<CategoryRow>, ServerFnError>,
    rename: ServerAction<RenameCategory>,
    delete: ServerAction<DeleteCategory>,
) -> AnyView {
    match rows {
        Ok(rows) if rows.is_empty() => view! { <p>{t("admin.categories.empty")}</p> }.into_any(),
        Ok(rows) => view! {
            <div class="admin-list">
                {rows
                    .into_iter()
                    .map(|row| category_card(row, rename, delete))
                    .collect_view()}
            </div>
        }
        .into_any(),
        Err(error) => error_notice(&message(&error)),
    }
}

fn category_card(
    row: CategoryRow,
    rename: ServerAction<RenameCategory>,
    delete: ServerAction<DeleteCategory>,
) -> impl IntoView {
    let id = row.id;
    view! {
        <section class="admin-row">
            <strong class="admin-row-title">{row.name.clone()}</strong>
            <ActionForm action=rename>
                <input type="hidden" name="id" value=id/>
                <label>{t("admin.label.name")} <input name="name" value=row.name required/></label>
                <button>{t("admin.button.rename")}</button>
            </ActionForm>
            <ActionForm action=delete>
                <input type="hidden" name="id" value=id/>
                <button class="danger">{t("admin.button.delete")}</button>
            </ActionForm>
        </section>
    }
}
