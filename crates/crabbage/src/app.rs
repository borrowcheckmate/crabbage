use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use crate::admin::{CategoriesPage, IngredientPage, IngredientsPage, ReviewPage, SectionsPage};
use crate::i18n::{current_language, t};
use crate::plan_page::PlanPage;
use crate::recipes::{
    CookPage, RecipeEditPage, RecipeImportPage, RecipeNewPage, RecipePage, RecipesPage,
};
use crate::shopping_page::ShoppingPage;
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
                    <Route path=path!("/admin/ingredients/:id") view=IngredientPage/>
                    <Route path=path!("/admin/sections") view=SectionsPage/>
                    <Route path=path!("/admin/categories") view=CategoriesPage/>
                    <Route path=path!("/admin/review") view=ReviewPage/>
                    <Route path=path!("/recipes") view=RecipesPage/>
                    <Route path=path!("/recipes/new") view=RecipeNewPage/>
                    <Route path=path!("/recipes/import") view=RecipeImportPage/>
                    <Route path=path!("/recipes/:id") view=RecipePage/>
                    <Route path=path!("/recipes/:id/cook") view=CookPage/>
                    <Route path=path!("/recipes/:id/edit") view=RecipeEditPage/>
                    <Route path=path!("/shopping") view=ShoppingPage/>
                    <Route path=path!("/plan") view=PlanPage/>
                </Routes>
                <form method="post" action="/auth/logout">
                    <button class="secondary">{t("app.sign_out")}</button>
                </form>
            </main>
        </Router>
    }
}

/// The home page: the recipes for everyone, and the admin pages for admins.
#[component]
fn Home() -> impl IntoView {
    view! {
        <h1>{t("app.title")}</h1>
        <Suspense fallback=|| ()>
            {move || Suspend::new(async move {
                let is_admin = matches!(current_user().await, Ok(user) if user.is_admin);
                view! {
                    <nav class="nav">
                        <a href="/recipes">{t("recipes.nav")}</a>
                        <a href="/shopping">{t("shopping.nav")}</a>
                        <a href="/plan">{t("plan.nav")}</a>
                        {is_admin.then(|| view! {
                            <a href="/admin/ingredients">{t("admin.nav.ingredients")}</a>
                            <a href="/admin/sections">{t("admin.nav.sections")}</a>
                            <a href="/admin/categories">{t("admin.nav.categories")}</a>
                            <a href="/admin/review">{t("admin.nav.review")}</a>
                        })}
                    </nav>
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
