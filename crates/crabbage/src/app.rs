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
                    <a class="brand" href="/">{t("app.title")}</a>
                    <AccountMenu/>
                </header>
                <div class="shell-layout">
                    <PrimaryNavigation/>
                    <main id="main-content">
                        <NoHouseholdNotice/>
                        <Routes fallback=|| ()>
                            <Route path=path!("/") view=Home/>
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
            {user.as_ref().and_then(|u| u.active_household.as_ref()).map(|name| view! { <span class="household-name">{name.clone()}</span> })}
            <details class="account-menu">
                <summary>{user.as_ref().map_or_else(|| t("app.account").to_owned(), |u| u.name.clone())}</summary>
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
            <a href="/" aria-current=move || (location.pathname.get() == "/").then_some("page")><svg aria-hidden="true" viewBox="0 0 24 24"><path d="m3 10 9-7 9 7v10a1 1 0 0 1-1 1h-6v-7h-4v7H4a1 1 0 0 1-1-1z"/></svg>{t("home.nav")}</a>
            <a href="/recipes" aria-current=move || location.pathname.get().starts_with("/recipes").then_some("page")><svg aria-hidden="true" viewBox="0 0 24 24"><path d="M4 5h16v15H4zM8 3v4m8-4v4M4 10h16"/></svg>{t("recipes.nav")}</a>
            <a href="/plan" aria-current=move || location.pathname.get().starts_with("/plan").then_some("page")><svg aria-hidden="true" viewBox="0 0 24 24"><rect x="3" y="4" width="18" height="17" rx="2"/><path d="M8 2v4m8-4v4M3 9h18m-13 4h2m4 0h2m-8 4h2m4 0h2"/></svg>{t("plan.nav")}</a>
            <a href="/shopping" aria-current=move || location.pathname.get().starts_with("/shopping").then_some("page")><svg aria-hidden="true" viewBox="0 0 24 24"><path d="M4 5h16l-1.5 15h-13zM9 9V7a3 3 0 0 1 6 0v2"/></svg>{t("shopping.nav")}</a>
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

/// The home page: the recipes for everyone, and the admin pages for admins.
#[component]
fn Home() -> impl IntoView {
    view! {
        <h1>{t("home.nav")}</h1>
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
