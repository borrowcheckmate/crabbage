use std::time::Duration;

use crabbage_import::web::{Draft, ImportError, extract};

const PLAIN: &str = include_str!("../fixtures/web/plain.html");
const GRAPH: &str = include_str!("../fixtures/web/graph.html");
const TYPE_ARRAY: &str = include_str!("../fixtures/web/type_array.html");
const SECTIONS: &str = include_str!("../fixtures/web/sections.html");
const STRING_INSTRUCTIONS: &str = include_str!("../fixtures/web/string_instructions.html");
const ARTICLE: &str = include_str!("../fixtures/web/article.html");
const BROKEN_THEN_VALID: &str = include_str!("../fixtures/web/broken_then_valid.html");
const NO_JSON_LD: &str = include_str!("../fixtures/web/no_json_ld.html");

fn script(json: &str) -> String {
    format!(r#"<script type="application/ld+json">{json}</script>"#)
}

fn recipe(extra: &str) -> String {
    script(&format!(r#"{{"@type": "Recipe", "name": "Test"{extra}}}"#))
}

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_owned()).collect()
}

#[test]
fn a_plain_recipe_gives_every_field() {
    let draft = extract(PLAIN).expect("plain recipe");
    assert_eq!(
        draft,
        Draft {
            title: "Lemon Pancakes".to_owned(),
            servings: Some(4),
            image: Some("https://example.com/images/pancakes.jpg".to_owned()),
            lines: strings(&["2 eggs", "200 ml milk", "150 g flour", "1 lemon, zest only"]),
            steps: strings(&["Whisk the eggs and milk.", "Add the flour and zest."]),
            prep_time: Some(Duration::from_mins(15)),
            cook_time: Some(Duration::from_mins(90)),
            total_time: Some(Duration::from_mins(105)),
        }
    );
}

#[test]
fn the_recipe_is_found_inside_a_graph() {
    let draft = extract(GRAPH).expect("graph recipe");
    assert_eq!(draft.title, "Carrot soup");
    assert_eq!(draft.servings, Some(6));
    assert_eq!(draft.lines, strings(&["500 g carrots", "1 l stock"]));
    assert_eq!(
        draft.steps,
        strings(&["Roast the carrots.", "Blend with the stock."])
    );
}

#[test]
fn a_type_list_and_an_image_object_are_read() {
    let draft = extract(TYPE_ARRAY).expect("type list recipe");
    assert_eq!(draft.title, "Sourdough");
    assert_eq!(draft.servings, Some(4));
    assert_eq!(
        draft.image.as_deref(),
        Some("https://example.com/img/bread.jpg")
    );
    assert_eq!(draft.lines, strings(&["500 g bread flour"]));
}

#[test]
fn sectioned_instructions_flatten_in_order() {
    let draft = extract(SECTIONS).expect("sectioned recipe");
    assert_eq!(
        draft.steps,
        strings(&[
            "Mix flour and water.",
            "Knead for ten minutes.",
            "Bake at 220 C."
        ])
    );
    assert_eq!(draft.prep_time, Some(Duration::from_mins(20)));
    assert_eq!(draft.cook_time, None);
    assert_eq!(draft.total_time, Some(Duration::from_secs(90)));
}

#[test]
fn string_instructions_split_on_lines_and_a_range_yield_takes_its_first_number() {
    let draft = extract(STRING_INSTRUCTIONS).expect("string instructions");
    assert_eq!(
        draft.steps,
        strings(&["Soak the beans.", "Boil for 25 minutes.", "Drain."])
    );
    assert_eq!(draft.servings, Some(4));
    assert_eq!(
        draft.image.as_deref(),
        Some("https://example.com/img/beans.jpg")
    );
}

#[test]
fn yields_take_the_first_integer_in_any_shape() {
    for (fields, servings) in [
        (r#", "recipeYield": 4"#, Some(4)),
        (r#", "recipeYield": "Serves 4""#, Some(4)),
        (r#", "recipeYield": "Makes 12 cookies""#, Some(12)),
        (r#", "recipeYield": ["4 servings", "8"]"#, Some(4)),
        (r#", "recipeYield": ["no number"]"#, None),
        (r#", "recipeYield": 0"#, None),
        (r#", "recipeYield": "about four""#, None),
        (r#", "recipeYield": {"value": 4}"#, None),
        ("", None),
    ] {
        let draft = extract(&recipe(fields)).expect(fields);
        assert_eq!(draft.servings, servings, "{fields}");
    }
}

#[test]
fn images_are_read_from_strings_arrays_and_objects() {
    for (fields, image) in [
        (
            r#", "image": "https://example.com/a.jpg""#,
            Some("https://example.com/a.jpg"),
        ),
        (
            r#", "image": ["https://example.com/b.jpg"]"#,
            Some("https://example.com/b.jpg"),
        ),
        (
            r#", "image": {"url": "https://example.com/c.jpg"}"#,
            Some("https://example.com/c.jpg"),
        ),
        (r#", "image": "  ""#, None),
        (r#", "image": 7"#, None),
        ("", None),
    ] {
        let draft = extract(&recipe(fields)).expect(fields);
        assert_eq!(draft.image.as_deref(), image, "{fields}");
    }
}

#[test]
fn lines_and_steps_accept_single_strings_and_skip_blank_entries() {
    let draft = extract(&recipe(
        r#", "recipeIngredient": "1 onion\n\n2 carrots", "recipeInstructions": [
            "Chop.", "  ", {"@type": "HowToStep", "name": "Fry the onion."},
            {"@type": "HowToStep", "text": "", "name": "Stir."}, 5, null,
            {"@type": "HowToSection", "name": "Empty", "itemListElement": []}
        ]"#,
    ))
    .expect("single string lines");
    assert_eq!(draft.lines, strings(&["1 onion", "2 carrots"]));
    assert_eq!(draft.steps, strings(&["Chop.", "Fry the onion.", "Stir."]));
}

#[test]
fn a_single_step_object_and_unlisted_shapes_are_read() {
    let single = extract(&recipe(
        r#", "recipeInstructions": {"@type": "HowToStep", "text": "Serve warm."}, "recipeIngredient": 3"#,
    ))
    .expect("single step");
    assert_eq!(single.steps, strings(&["Serve warm."]));
    assert_eq!(single.lines, Vec::<String>::new());
}

#[test]
fn durations_are_read_as_iso_8601_and_unparseable_ones_are_none() {
    for (text, expected) in [
        ("PT1H30M", Some(Duration::from_mins(90))),
        ("PT45M", Some(Duration::from_mins(45))),
        ("P0DT20M", Some(Duration::from_mins(20))),
        ("P1DT2H", Some(Duration::from_hours(26))),
        ("PT90S", Some(Duration::from_secs(90))),
        ("PT", None),
        ("P", None),
        ("P1M", None),
        ("PT1.5H", None),
        ("PT10", None),
        ("PT1H1H", Some(Duration::from_hours(2))),
        ("about an hour", None),
    ] {
        let draft = extract(&recipe(&format!(r#", "prepTime": "{text}""#))).expect(text);
        assert_eq!(draft.prep_time, expected, "{text}");
    }
}

#[test]
fn an_invalid_json_script_is_skipped() {
    let draft = extract(BROKEN_THEN_VALID).expect("second script");
    assert_eq!(draft.title, "Plum crumble");
    assert_eq!(draft.servings, Some(6));
}

#[test]
fn a_page_with_only_an_article_has_no_recipe() {
    assert_eq!(extract(ARTICLE), Err(ImportError::NoRecipe));
}

#[test]
fn a_page_without_json_ld_has_no_recipe() {
    assert_eq!(extract(NO_JSON_LD), Err(ImportError::NoRecipe));
}

#[test]
fn a_recipe_without_a_name_is_not_a_recipe() {
    let html = script(r#"{"@type": "Recipe", "name": "  ", "recipeIngredient": ["1 egg"]}"#);
    assert_eq!(extract(&html), Err(ImportError::NoRecipe));
    let nested = script(r#"{"@type": "Recipe", "recipeIngredient": ["1 egg"]}"#);
    assert_eq!(extract(&nested), Err(ImportError::NoRecipe));
}

#[test]
fn a_recipe_without_a_type_is_not_a_recipe() {
    let html = script(r#"{"@type": "Thing", "name": "Not a recipe"}"#);
    assert_eq!(extract(&html), Err(ImportError::NoRecipe));
}

#[test]
fn script_tags_are_found_regardless_of_case_attributes_and_other_scripts() {
    let html = concat!(
        "<script>var recipe = {\"@type\": \"Recipe\", \"name\": \"Decoy\"};</script>",
        "<SCRIPT id=\"schema\" TYPE='application/ld+json'>",
        "{\"@type\": \"Recipe\", \"name\": \"Pear tart\"}",
        "</SCRIPT>",
    );
    assert_eq!(
        extract(html).map(|draft| draft.title).ok().as_deref(),
        Some("Pear tart")
    );
}

#[test]
fn an_unclosed_script_is_ignored() {
    let html = "<script type=\"application/ld+json\">{\"@type\": \"Recipe\", \"name\": \"Open\"}";
    assert_eq!(extract(html), Err(ImportError::NoRecipe));
    assert_eq!(
        extract("<script type=\"application/ld+json\""),
        Err(ImportError::NoRecipe)
    );
}
