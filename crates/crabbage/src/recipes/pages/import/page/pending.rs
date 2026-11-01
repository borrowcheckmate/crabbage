use leptos::prelude::*;

use crate::i18n::t;

#[component]
pub(super) fn ImportPending(message: &'static str) -> impl IntoView {
    view! {
        <div class="import-pending" role="status">
            <span>{message}</span>
            <div class="import-progress-bar" role="progressbar" aria-label=t("import.progress_label")></div>
        </div>
    }
}

#[component]
pub(super) fn ImportProgress(current: RwSignal<u8>) -> impl IntoView {
    let stages = [
        "import.stage.source",
        "import.stage.read",
        "import.stage.review",
    ];
    view! {
        <ol class="import-progress" aria-label=t("import.progress_label")>
            {stages.into_iter().enumerate().map(|(index, key)| view! {
            <li class:current=move || index == usize::from(current.get()) class:complete=move || index < usize::from(current.get()) aria-current=move || (index == usize::from(current.get())).then_some("step")>
                    <span>{move || if index < usize::from(current.get()) { view! { <svg aria-hidden="true" viewBox="0 0 24 24"><path d="m5 12 4 4L19 6"/></svg> }.into_any() } else { (index + 1).to_string().into_any() }}</span>{t(key)}
                </li>
            }).collect_view()}
        </ol>
    }
}
