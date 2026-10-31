use std::time::Duration;

use serde_json::{Map, Value};

use super::duration;
use super::{Draft, ImportError};

/// Reads the first schema.org `Recipe` in the page's JSON-LD. Scripts that are not valid JSON and
/// recipes without a name are skipped.
///
/// # Errors
///
/// [`ImportError::NoRecipe`] when no usable recipe is found.
pub fn extract(html: &str) -> Result<Draft, ImportError> {
    json_ld_scripts(html)
        .into_iter()
        .filter_map(|script| serde_json::from_str::<Value>(script).ok())
        .find_map(|value| recipe_node(&value).and_then(draft))
        .ok_or(ImportError::NoRecipe)
}

fn json_ld_scripts(html: &str) -> Vec<&str> {
    let lower = html.to_ascii_lowercase();
    let mut scripts = Vec::new();
    let mut from = 0;
    while let Some(start) = find(&lower, from, "<script") {
        let Some(open_end) = find(&lower, start, ">").map(|end| end + 1) else {
            break;
        };
        let Some(close) = find(&lower, open_end, "</script") else {
            break;
        };
        let is_json_ld = lower
            .get(start..open_end)
            .is_some_and(|tag| tag.contains("application/ld+json"));
        if let Some(script) = html.get(open_end..close).filter(|_| is_json_ld) {
            scripts.push(script);
        }
        from = close;
    }
    scripts
}

fn find(text: &str, from: usize, needle: &str) -> Option<usize> {
    text.get(from..)?.find(needle).map(|index| from + index)
}

fn recipe_node(value: &Value) -> Option<&Map<String, Value>> {
    match value {
        Value::Array(items) => items.iter().find_map(recipe_node),
        Value::Object(object) if is_recipe(object) => Some(object),
        Value::Object(object) => object.get("@graph").and_then(recipe_node),
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => None,
    }
}

fn is_recipe(object: &Map<String, Value>) -> bool {
    object.get("@type").is_some_and(has_recipe_type) && title(object).is_some()
}

fn has_recipe_type(kind: &Value) -> bool {
    match kind {
        Value::String(name) => name.as_str() == "Recipe",
        Value::Array(kinds) => kinds.iter().any(has_recipe_type),
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::Object(_) => false,
    }
}

fn draft(recipe: &Map<String, Value>) -> Option<Draft> {
    Some(Draft {
        title: title(recipe)?,
        servings: recipe.get("recipeYield").and_then(servings),
        image: recipe.get("image").and_then(image),
        lines: recipe
            .get("recipeIngredient")
            .map_or_else(Vec::new, ingredients),
        steps: recipe
            .get("recipeInstructions")
            .map_or_else(Vec::new, steps),
        prep_time: time(recipe, "prepTime"),
        cook_time: time(recipe, "cookTime"),
        total_time: time(recipe, "totalTime"),
    })
}

fn title(recipe: &Map<String, Value>) -> Option<String> {
    recipe
        .get("name")
        .and_then(Value::as_str)
        .and_then(non_empty)
}

fn time(recipe: &Map<String, Value>, key: &str) -> Option<Duration> {
    recipe
        .get(key)
        .and_then(Value::as_str)
        .and_then(duration::parse)
}

fn servings(value: &Value) -> Option<u32> {
    match value {
        Value::Number(number) => number.as_u64().and_then(positive),
        Value::String(text) => first_integer(text),
        Value::Array(items) => items.iter().find_map(servings),
        Value::Null | Value::Bool(_) | Value::Object(_) => None,
    }
}

fn first_integer(text: &str) -> Option<u32> {
    text.chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse::<u64>()
        .ok()
        .and_then(positive)
}

fn positive(number: u64) -> Option<u32> {
    u32::try_from(number).ok().filter(|n| *n > 0)
}

fn image(value: &Value) -> Option<String> {
    match value {
        Value::String(url) => non_empty(url),
        Value::Array(items) => items.iter().find_map(image),
        Value::Object(object) => object.get("url").and_then(image),
        Value::Null | Value::Bool(_) | Value::Number(_) => None,
    }
}

fn ingredients(value: &Value) -> Vec<String> {
    match value {
        Value::Array(items) => items
            .iter()
            .filter_map(Value::as_str)
            .filter_map(non_empty)
            .collect(),
        Value::String(text) => lines(text),
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::Object(_) => Vec::new(),
    }
}

fn steps(value: &Value) -> Vec<String> {
    match value {
        Value::String(text) => lines(text),
        Value::Array(items) => items.iter().flat_map(steps).collect(),
        Value::Object(object) => object
            .get("itemListElement")
            .map_or_else(|| step_text(object), steps),
        Value::Null | Value::Bool(_) | Value::Number(_) => Vec::new(),
    }
}

fn step_text(object: &Map<String, Value>) -> Vec<String> {
    ["text", "name"]
        .iter()
        .find_map(|key| object.get(*key).and_then(Value::as_str).and_then(non_empty))
        .into_iter()
        .collect()
}

fn lines(text: &str) -> Vec<String> {
    text.lines().filter_map(non_empty).collect()
}

fn non_empty(text: &str) -> Option<String> {
    let trimmed = text.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}
