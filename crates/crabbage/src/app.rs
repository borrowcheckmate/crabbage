use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use crate::admin::{IngredientsPage, ReviewPage, SectionsPage};
use crate::i18n::{current_language, t};
use crate::user::current_user;

/// The root component of the app: the routes and the sign-out button.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    view! {
        <Title text=t("app.title")/>
        <Router>
            <main>
                <Routes fallback=|| ()>
                    <Route path=path!("/") view=Home/>
                    <Route path=path!("/admin/ingredients") view=IngredientsPage/>
                    <Route path=path!("/admin/sections") view=SectionsPage/>
                    <Route path=path!("/admin/review") view=ReviewPage/>
                </Routes>
                <form method="post" action="/auth/logout">
                    <button class="secondary">{t("app.sign_out")}</button>
                </form>
            </main>
        </Router>
    }
}

/// The home page. Admins also get links to the admin pages.
#[component]
fn Home() -> impl IntoView {
    view! {
        <h1>{t("app.title")}</h1>
        <Suspense fallback=|| ()>
            {move || Suspend::new(async move {
                match current_user().await {
                    Ok(user) if user.is_admin => view! {
                        <nav class="nav">
                            <a href="/admin/ingredients">{t("admin.nav.ingredients")}</a>
                            <a href="/admin/sections">{t("admin.nav.sections")}</a>
                            <a href="/admin/review">{t("admin.nav.review")}</a>
                        </nav>
                    }
                    .into_any(),
                    Ok(_) | Err(_) => ().into_any(),
                }
            })}
        </Suspense>
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
                <Stylesheet id="leptos" href="/pkg/crabbage.css"/>
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
