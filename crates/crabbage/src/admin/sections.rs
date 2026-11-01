//! Store sections: create, rename, delete and move up or down.

use leptos::form::ActionForm;
use leptos::prelude::*;

use crate::notice::{error_notice, failure, message};
use leptos::prelude::ServerFnError;

use crate::catalog::SectionRow;
use crate::catalog::sections::{
    CreateSection, DeleteSection, MoveSection, RenameSection, list_sections,
};
use crate::i18n::t;
use crate::user::current_user;

type SectionPageData = (
    Result<crate::user::SignedInUser, ServerFnError>,
    Option<Result<Vec<SectionRow>, ServerFnError>>,
);

/// The store sections page. Admins see the forms and the rows, everyone else a message. The data is
/// loaded inside one suspense.
#[component]
pub(crate) fn SectionsPage() -> impl IntoView {
    let create = ServerAction::<CreateSection>::new();
    let rename = ServerAction::<RenameSection>::new();
    let delete = ServerAction::<DeleteSection>::new();
    let move_section = ServerAction::<MoveSection>::new();
    let data = Resource::new(
        move || {
            (
                create.version().get(),
                rename.version().get(),
                delete.version().get(),
                move_section.version().get(),
            )
        },
        |_| load_sections(),
    );
    view! {
        <Suspense fallback=|| view! { <p class="state-notice" role="status">{t("admin.loading")}</p> }>
            {move || {
                Suspend::new(async move { section_page(data.await, create, rename, delete, move_section) })
            }}
        </Suspense>
    }
}

async fn load_sections() -> (
    Result<crate::user::SignedInUser, ServerFnError>,
    Option<Result<Vec<SectionRow>, ServerFnError>>,
) {
    let user = current_user().await;
    let sections = if user.as_ref().is_ok_and(|user| user.is_admin) {
        Some(list_sections().await)
    } else {
        None
    };
    (user, sections)
}

fn section_page(
    (user, sections): SectionPageData,
    create: ServerAction<CreateSection>,
    rename: ServerAction<RenameSection>,
    delete: ServerAction<DeleteSection>,
    move_section: ServerAction<MoveSection>,
) -> AnyView {
    match user {
        Ok(user) if user.is_admin => view! {
            <section class="admin-page">
            <h1>{t("admin.sections.heading")}</h1>
            <section class="admin-card admin-create"><ActionForm action=create>
                <label>{t("admin.label.name")} <input name="name" required/></label>
                <button>{t("admin.button.add")}</button>
            </ActionForm></section>
            {move || create.value().with(|value| failure(value.as_ref()))}
            {move || rename.value().with(|value| failure(value.as_ref()))}
            {move || delete.value().with(|value| failure(value.as_ref()))}
            {move || move_section.value().with(|value| failure(value.as_ref()))}
            {section_rows(sections, rename, delete, move_section)}
            </section>
        }
        .into_any(),
        Ok(_) | Err(_) => error_notice(t("admin.not_allowed")),
    }
}

/// The rows of the section list, or the message that explains why they are missing.
fn section_rows(
    sections: Option<Result<Vec<SectionRow>, ServerFnError>>,
    rename: ServerAction<RenameSection>,
    delete: ServerAction<DeleteSection>,
    move_section: ServerAction<MoveSection>,
) -> AnyView {
    match sections {
        Some(Ok(rows)) if rows.is_empty() => {
            view! { <p>{t("admin.sections.empty")}</p> }.into_any()
        }
        Some(Ok(rows)) => view! {
            <div class="admin-list">
                {rows
                    .into_iter()
                    .map(|row| view! { <SectionRow_ row=row rename=rename delete=delete move_section=move_section/> })
                    .collect_view()}
            </div>
        }
        .into_any(),
        Some(Err(error)) => error_notice(&message(&error)),
        None => ().into_any(),
    }
}

#[component]
fn SectionRow_(
    row: SectionRow,
    rename: ServerAction<RenameSection>,
    delete: ServerAction<DeleteSection>,
    move_section: ServerAction<MoveSection>,
) -> impl IntoView {
    let id = row.id;
    let up = (row.position - 1).to_string();
    let down = (row.position + 1).to_string();
    view! {
        <section class="admin-row">
            <strong class="admin-row-title">{row.name.clone()}</strong>
            <ActionForm action=rename>
                <input type="hidden" name="id" value=id/>
                <label>{t("admin.label.name")} <input name="name" value=row.name.clone() required/></label>
                <button>{t("admin.button.rename")}</button>
            </ActionForm>
            <div class="row admin-row-actions">
                <ActionForm action=move_section>
                    <input type="hidden" name="id" value=id/>
                    <input type="hidden" name="position" value=up/>
                    <button class="secondary">{t("admin.button.up")}</button>
                </ActionForm>
                <ActionForm action=move_section>
                    <input type="hidden" name="id" value=id/>
                    <input type="hidden" name="position" value=down/>
                    <button class="secondary">{t("admin.button.down")}</button>
                </ActionForm>
            </div>
            <ActionForm action=delete>
                <input type="hidden" name="id" value=id/>
                <button class="danger">{t("admin.button.delete")}</button>
            </ActionForm>
        </section>
    }
}
