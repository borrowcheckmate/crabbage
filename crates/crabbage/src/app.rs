use leptos::prelude::*;
use leptos_meta::{MetaTags, Title, provide_meta_context};

/// The root component of the app.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    view! {
        <Title text="crabbage"/>
        <main>
            <h1>"crabbage"</h1>
        </main>
    }
}

/// The HTML document around [`App`], used for server rendering.
#[must_use]
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
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
