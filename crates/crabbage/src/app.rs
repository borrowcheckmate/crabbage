use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::hooks::use_location;
use leptos_router::path;

use crate::admin::{
    CategoriesPage, IngredientPage, IngredientsPage, MealiePage, ReviewPage, SectionsPage,
};
use crate::i18n::{current_language, t};
use crate::plan_page::PlanPage;
use crate::recipes::{
    CookPage, RecipeEditPage, RecipeImportPage, RecipeNewPage, RecipePage, RecipesPage,
};
use crate::shopping_page::ShoppingPage;
use crate::today_page::TodayPage;
use crate::user::current_user;

/// The root component of the app: the routes and the shared navigation.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    view! {
        <Title text=t("app.title")/>
        <Router>
            <a class="skip-link" href="#main-content">{t("app.skip")}</a>
            <div class="app-frame">
                <header class="topbar">
                    <a class="brand" href="/"><svg aria-hidden="true" viewBox="0 0 24 24"><path d="M7 20h10M10 20c5.5-2.5.8-6.4 3-10M9.5 9.4c1.1.8 1.8 2.2 2.3 3.7-2 .4-3.5.4-4.8-.3-1.2-.6-2.3-1.9-3-4.2 2.8-.5 4.4 0 5.5.8zM14.1 6a7 7 0 0 0-1.1 4c1.9-.1 3.3-.6 4.3-1.4 1-1 1.6-2.3 1.7-4.6-2.7.1-4 1-4.9 2z"/></svg>{t("app.title")}</a>
                    <AccountMenu/>
                </header>
                <div class="shell-layout">
                    <PrimaryNavigation/>
                    <main id="main-content">
                        <NoHouseholdNotice/>
                        <Routes fallback=NotFound>
                            <Route path=path!("/") view=TodayPage/>
                            <Route path=path!("/admin/ingredients") view=IngredientsPage/>
                            <Route path=path!("/admin/ingredients/:id") view=IngredientPage/>
                            <Route path=path!("/admin/sections") view=SectionsPage/>
                            <Route path=path!("/admin/categories") view=CategoriesPage/>
                            <Route path=path!("/admin/review") view=ReviewPage/>
                            <Route path=path!("/admin/mealie") view=MealiePage/>
                            <Route path=path!("/recipes") view=RecipesPage/>
                            <Route path=path!("/recipes/new") view=RecipeNewPage/>
                            <Route path=path!("/recipes/import") view=RecipeImportPage/>
                            <Route path=path!("/recipes/:id") view=RecipePage/>
                            <Route path=path!("/recipes/:id/cook") view=CookPage/>
                            <Route path=path!("/recipes/:id/edit") view=RecipeEditPage/>
                            <Route path=path!("/shopping") view=ShoppingPage/>
                            <Route path=path!("/plan") view=PlanPage/>
                        </Routes>
                    </main>
                </div>
                <MobileNavigation/>
            </div>
        </Router>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <section class="state-card" role="status">
            <h1>{t("system.not_found.heading")}</h1>
            <p>{t("system.not_found.message")}</p>
            <a class="button-link" href="/">{t("system.not_found.home")}</a>
        </section>
    }
}

#[component]
fn AccountMenu() -> impl IntoView {
    view! {
        <Suspense fallback=|| ()>
            {move || Suspend::new(async move {account_view(current_user().await.ok())})}
        </Suspense>
    }
}

fn account_view(user: Option<crate::user::SignedInUser>) -> impl IntoView {
    view! {
        <div class="account-area">
            {user.as_ref().and_then(|u| u.active_household.as_ref()).map(|name| view! { <span class="household"><svg aria-hidden="true" viewBox="0 0 24 24"><path d="M15 21v-8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v8M3 10a2 2 0 0 1 .7-1.5l7-6a2 2 0 0 1 2.6 0l7 6A2 2 0 0 1 21 10v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/></svg><span class="household-name">{name.clone()}</span></span> })}
            <details class="account-menu">
                <summary aria-label=user.as_ref().map_or_else(|| t("app.account").to_owned(), |u| u.name.clone())><span class="avatar" aria-hidden="true">{user.as_ref().and_then(|u| u.name.chars().next()).map_or_else(String::new, |c| c.to_uppercase().collect())}</span></summary>
                {user.map(|u| view! { <div class="account-panel">
                    <HouseholdSwitcher user=u.clone()/>
                    <AdminLinks is_admin=u.is_admin/>
                    <form method="post" action="/auth/logout"><button class="quiet">{t("app.sign_out")}</button></form>
                </div> })}
            </details>
        </div>
    }
}

#[component]
fn HouseholdSwitcher(user: crate::user::SignedInUser) -> impl IntoView {
    (user.households.len() > 1).then(|| view! {
        <form method="post" action="/household">
            <label>{t("app.switch_household")}<select name="household_id">
                {user.households.iter().map(|(id, name)| view! {
                    <option value=id.to_string() selected=user.active_household.as_deref() == Some(name)>{name.clone()}</option>
                }).collect_view()}
            </select></label>
            <button class="quiet">{t("app.switch")}</button>
        </form>
    })
}

#[component]
fn AdminLinks(is_admin: bool) -> impl IntoView {
    is_admin.then(|| {
        view! {
            <nav aria-label=t("admin.nav.label")>
                <a href="/admin/ingredients">{t("admin.nav.ingredients")}</a>
                <a href="/admin/sections">{t("admin.nav.sections")}</a>
                <a href="/admin/categories">{t("admin.nav.categories")}</a>
                <a href="/admin/review">{t("admin.nav.review")}</a>
                <a href="/admin/mealie">{t("admin.nav.mealie")}</a>
            </nav>
        }
    })
}

#[component]
fn PrimaryNavigation() -> impl IntoView {
    navigation("primary-nav")
}

#[component]
fn MobileNavigation() -> impl IntoView {
    navigation("mobile-nav")
}

fn navigation(class: &'static str) -> impl IntoView {
    let location = use_location();
    view! {
        <nav class=class aria-label=t("app.navigation")>
            <a href="/" aria-current=move || (location.pathname.get() == "/").then_some("page")><svg aria-hidden="true" viewBox="0 0 24 24"><circle cx="12" cy="12" r="4"/><path d="M12 2v2m0 16v2M4.9 4.9l1.4 1.4m11.4 11.4 1.4 1.4M2 12h2m16 0h2M6.3 17.7l-1.4 1.4M19.1 4.9l-1.4 1.4"/></svg><span>{t("home.nav")}</span></a>
            <a href="/recipes" aria-current=move || location.pathname.get().starts_with("/recipes").then_some("page")><svg aria-hidden="true" viewBox="0 0 24 24"><path d="M12 7v14M3 18a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1h5a4 4 0 0 1 4 4 4 4 0 0 1 4-4h5a1 1 0 0 1 1 1v13a1 1 0 0 1-1 1h-6a3 3 0 0 0-3 3 3 3 0 0 0-3-3z"/></svg><span>{t("recipes.nav")}</span></a>
            <a href="/plan" aria-current=move || location.pathname.get().starts_with("/plan").then_some("page")><svg aria-hidden="true" viewBox="0 0 24 24"><rect x="3" y="4" width="18" height="17" rx="2"/><path d="M8 2v4m8-4v4M3 9h18m-13 4h2m4 0h2m-8 4h2m4 0h2"/></svg><span>{t("plan.nav")}</span></a>
            <a href="/shopping" aria-current=move || location.pathname.get().starts_with("/shopping").then_some("page")><svg aria-hidden="true" viewBox="0 0 24 24"><path d="m15 11-1 9m5-9-4-7M2 11h20m-18.5 0 1.6 7.4a2 2 0 0 0 2 1.6h9.8a2 2 0 0 0 2-1.6l1.7-7.4M4.5 15.5h15M5 11l4-7m0 7 1 9"/></svg><span>{t("shopping.nav")}</span></a>
        </nav>
    }
}

#[component]
fn NoHouseholdNotice() -> impl IntoView {
    let location = use_location();
    view! {
        <Show when=move || location.pathname.get().starts_with("/plan") || location.pathname.get().starts_with("/shopping")>
            <Suspense fallback=|| ()>
                {move || Suspend::new(async move {
                    current_user().await.is_ok_and(|user| user.active_household.is_none())
                        .then(|| view! { <p class="notice">{t("app.no_household")}</p> })
                })}
            </Suspense>
        </Show>
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
