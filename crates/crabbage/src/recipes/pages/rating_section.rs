//! The rating of a recipe and the controls for the signed-in user: five star buttons, filled up to
//! their own rating and that one pressed, and a button to add the recipe to their favourites or remove it. The
//! buttons are plain forms that post to the rating and favourite routes.

use leptos::prelude::*;

use crate::i18n::{t, t_with};
use crate::recipes::rating_text::rating_text;

/// The rating section of recipe `id`, with its average and count, the signed-in user's own rating
/// and whether the recipe is their favourite.
pub(crate) fn rating_section(
    id: i64,
    average: Option<f64>,
    rating_count: i64,
    mine: Option<i64>,
    favourite: bool,
) -> AnyView {
    let rating_href = format!("/recipes/{id}/rating");
    let favourite_href = format!("/recipes/{id}/favourite");
    let summary = rating_text(average, rating_count);
    let stars = (1..=5_i64)
        .map(|value| {
            let label = t_with("recipe.rating.choose", &[("stars", &value.to_string())]);
            view! {
                <button name="stars" value=value.to_string() aria-label=label aria-pressed=(mine == Some(value)).to_string()>
                    {if mine.is_some_and(|stars| value <= stars) { t("recipe.rating.star") } else { t("recipe.rating.star_empty") }}
                </button>
            }
        })
        .collect_view();
    let toggle = if favourite {
        t("recipe.favourite.remove")
    } else {
        t("recipe.favourite.add")
    };
    view! {
        <section class="card">
            <p>{summary}</p>
            <form class="row rating" method="post" action=rating_href aria-label=t("recipe.rating.yours")>
                {stars}
            </form>
            <form method="post" action=favourite_href>
                <input type="hidden" name="on" value=(!favourite).to_string()/>
                <button class="secondary">{toggle}</button>
            </form>
        </section>
    }
    .into_any()
}
