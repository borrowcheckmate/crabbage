use leptos::prelude::*;
use leptos_meta::{MetaTags, Title, provide_meta_context};

use crate::i18n::{current_language, t};

/// The root component of the app.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    view! {
        <Title text=t("app.title")/>
        <main>
            <h1>{t("app.title")}</h1>
            <form method="post" action="/auth/logout">
                <button>{t("app.sign_out")}</button>
            </form>
        </main>
    }
}

/// The HTML document around [`App`], used for server rendering. Its `lang` attribute is the
/// language provided to the render.
#[must_use]
pub fn shell(options: LeptosOptions) -> impl IntoView {
    let lang = current_language().code();
    view! {
        <!DOCTYPE html>
        <html lang=lang>
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}
