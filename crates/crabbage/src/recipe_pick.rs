//! Reads what was typed in the plan's recipe field as a rest day or one recipe.

use crabbage_import::matching::normalise;

/// What the plan's recipe field names.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Pick {
    /// A rest day.
    Rest,
    /// The recipe with this id.
    Recipe(i64),
}

/// `rest` or the rest label pick a rest day. Otherwise the recipe whose title equals `text`, then
/// the recipe with `text` as its id, then the first recipe whose title holds every typed word in
/// any order. Case, accents and extra spaces are ignored. `None` when nothing matches.
pub(crate) fn pick(text: &str, rest_label: &str, recipes: &[(i64, String)]) -> Option<Pick> {
    let wanted = normalise(text);
    if wanted.is_empty() {
        return None;
    }
    if wanted == "rest" || wanted == normalise(rest_label) {
        return Some(Pick::Rest);
    }
    let titles: Vec<(i64, String)> = recipes
        .iter()
        .map(|(id, title)| (*id, normalise(title)))
        .collect();
    let words: Vec<&str> = wanted.split(' ').collect();
    titles
        .iter()
        .find(|(_, title)| *title == wanted)
        .map(|(id, _)| *id)
        .or_else(|| {
            wanted
                .parse::<i64>()
                .ok()
                .filter(|id| titles.iter().any(|(recipe, _)| recipe == id))
        })
        .or_else(|| {
            titles
                .iter()
                .find(|(_, title)| words.iter().all(|word| title.contains(word)))
                .map(|(id, _)| *id)
        })
        .map(Pick::Recipe)
}

#[cfg(test)]
mod tests {
    use super::{Pick, pick};

    fn recipes() -> Vec<(i64, String)> {
        vec![
            (1, "Crème brûlée".to_owned()),
            (2, "Pasta pesto".to_owned()),
            (3, "Pasta".to_owned()),
            (42, "Soep".to_owned()),
        ]
    }

    #[test]
    fn rest_is_picked_by_its_label_or_the_word_rest() {
        assert_eq!(pick("Rustdag", "Rustdag", &recipes()), Some(Pick::Rest));
        assert_eq!(pick(" rest ", "Rustdag", &recipes()), Some(Pick::Rest));
    }

    #[test]
    fn a_title_matches_without_case_accents_or_word_order() {
        assert_eq!(
            pick("creme brulee", "Rest day", &recipes()),
            Some(Pick::Recipe(1))
        );
        assert_eq!(
            pick("pesto  PASTA", "Rest day", &recipes()),
            Some(Pick::Recipe(2))
        );
        assert_eq!(pick("brûl", "Rest day", &recipes()), Some(Pick::Recipe(1)));
    }

    #[test]
    fn an_exact_title_beats_a_title_that_contains_it() {
        assert_eq!(pick("pasta", "Rest day", &recipes()), Some(Pick::Recipe(3)));
    }

    #[test]
    fn a_known_id_is_picked_and_unknown_text_or_ids_are_not() {
        assert_eq!(pick("42", "Rest day", &recipes()), Some(Pick::Recipe(42)));
        assert_eq!(pick("7", "Rest day", &recipes()), None);
        assert_eq!(pick("lasagne", "Rest day", &recipes()), None);
        assert_eq!(pick("  ", "Rest day", &recipes()), None);
    }
}
