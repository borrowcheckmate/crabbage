//! The average rating and the number of ratings in the words and decimal separator of the language.

use crate::i18n::{messages, t, t_with};

/// The rating of a recipe: `★ 4.5 (2 ratings)`, or `No ratings yet` when nobody rated it.
pub(crate) fn rating_text(average: Option<f64>, count: i64) -> String {
    let Some(average) = average else {
        return t("recipe.rating.none").to_owned();
    };
    let average = format!("{average:.1}").replace('.', messages().get("number.decimal"));
    let key = if count == 1 {
        "recipe.rating.one"
    } else {
        "recipe.rating.many"
    };
    t_with(key, &[("average", &average), ("count", &count.to_string())])
}
