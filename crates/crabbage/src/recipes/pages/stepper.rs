//! The servings stepper of the recipe page: a minus and a plus link that change the `servings` value
//! of the address, the count between them, and Reset once the count is not the recipe's own.

use leptos::prelude::*;

use crate::i18n::{t, t_with};

/// The stepper for `shown` servings of recipe `id`, whose own count is `original`. The minus link is
/// hidden at one serving and above the address limit of 99, the plus link at 99.
pub(crate) fn servings_stepper(id: i64, shown: i64, original: i64) -> impl IntoView {
    let less = (2..=99)
        .contains(&shown)
        .then(|| format!("/recipes/{id}?servings={}", shown.saturating_sub(1)));
    let more = (1..99)
        .contains(&shown)
        .then(|| format!("/recipes/{id}?servings={}", shown.saturating_add(1)));
    let reset = (shown != original).then(|| format!("/recipes/{id}"));
    view! {
        <div class="stepper row" role="group" aria-label=t("recipe.servings")>
            {less.map(|href| view! { <a href=href aria-label=t("recipe.servings.less")>{t("recipe.servings.minus")}</a> })}
            <span>{count_text(shown)}</span>
            {more.map(|href| view! { <a href=href aria-label=t("recipe.servings.more")>{t("recipe.servings.plus")}</a> })}
            {reset.map(|href| view! { <a href=href>{t("recipe.servings.reset")}</a> })}
        </div>
    }
}

fn count_text(count: i64) -> String {
    let key = if count == 1 {
        "recipe.servings.one"
    } else {
        "recipe.servings.many"
    };
    t_with(key, &[("count", &count.to_string())])
}
