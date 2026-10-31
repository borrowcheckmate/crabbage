use std::collections::HashMap;

use crabbage_domain::quantity::Unit;

use crate::lines::measure_of;
use crate::matching::{Candidate, match_ingredient, normalise};

use super::{Category, Food, Ingredient, MealieUnit, Recipe, Tag};

/// A food that should be created as an ingredient pending review.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewIngredient {
    /// The ingredient's preferred name.
    pub name: String,
    /// Other names accepted for the ingredient.
    pub aliases: Vec<String>,
}

/// The resolved destination of a Mealie food.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoodResolution {
    /// An ingredient already in the catalogue.
    Existing(i64),
    /// An entry in [`FoodPlan::new_ingredients`].
    New(usize),
}

/// Ingredient creation requests and Mealie food resolutions.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FoodPlan {
    /// New ingredients to create as pending review.
    pub new_ingredients: Vec<NewIngredient>,
    /// Mealie food ids mapped to existing or new ingredients.
    pub foods: HashMap<String, FoodResolution>,
}

impl FoodPlan {
    /// Resolves a Mealie food id.
    #[must_use]
    pub fn resolve(&self, id: &str) -> Option<FoodResolution> {
        self.foods.get(id).copied()
    }
}

/// A category's resolved destination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CategoryResolution {
    /// An existing category id.
    Existing(i64),
    /// A name to create as a new category.
    New(String),
}

/// Category resolutions in source order.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CategoryPlan {
    /// Each Mealie category or tag name, trimmed, with its resolution.
    pub categories: Vec<(String, CategoryResolution)>,
}

impl CategoryPlan {
    /// Resolves a category name of a mapped recipe, ignoring case and surrounding space.
    #[must_use]
    pub fn resolve(&self, name: &str) -> Option<&CategoryResolution> {
        let key = normalise(name);
        self.categories
            .iter()
            .find(|(known, _)| normalise(known) == key)
            .map(|(_, resolution)| resolution)
    }
}

/// A mapped ingredient line.
#[derive(Debug, Clone, PartialEq)]
pub struct MappedLine {
    /// The amount in `unit`, when known.
    pub amount: Option<f64>,
    /// The domain unit, when known.
    pub unit: Option<Unit>,
    /// The matched food destination.
    pub food: Option<FoodResolution>,
    /// A note associated with the line.
    pub note: Option<String>,
    /// The original line text supplied by Mealie.
    pub original: String,
    /// The source text for a line without a food reference.
    pub text: Option<String>,
}

/// A mapped instruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MappedStep {
    /// The instruction text.
    pub text: String,
}

/// A recipe translated from Mealie's data model.
#[derive(Debug, Clone, PartialEq)]
pub struct MappedRecipe {
    /// The Mealie recipe id, retained for idempotent persistence.
    pub mealie_id: String,
    /// The recipe title.
    pub title: String,
    /// The number of servings, when available.
    pub servings: Option<u32>,
    /// The Mealie image id, when the recipe has an image.
    pub image_id: Option<String>,
    /// The rounded crabbage rating.
    pub rating: Option<u8>,
    /// Category names from categories and tags.
    pub categories: Vec<String>,
    /// Instructions in order.
    pub steps: Vec<MappedStep>,
    /// Ingredient lines in order.
    pub lines: Vec<MappedLine>,
}

/// Matches Mealie foods against existing ingredients and plans new ingredients.
#[must_use]
pub fn map_foods(foods: &[Food], existing: &[Candidate<'_>]) -> FoodPlan {
    let mut plan = FoodPlan::default();
    let mut new_by_name = HashMap::new();
    for food in foods {
        let mut names = vec![food.name.as_str()];
        names.extend(food.aliases.iter().map(|alias| alias.name.as_str()));
        let matched = names
            .iter()
            .find_map(|name| match_ingredient(name, existing));
        let resolution = if let Some(id) = matched {
            FoodResolution::Existing(id)
        } else {
            let key = normalise(&food.name);
            let index = if let Some(index) = new_by_name.get(&key).copied() {
                index
            } else {
                let index = plan.new_ingredients.len();
                plan.new_ingredients.push(NewIngredient {
                    name: food.name.clone(),
                    aliases: Vec::new(),
                });
                new_by_name.insert(key, index);
                index
            };
            let Some(entry) = plan.new_ingredients.get_mut(index) else {
                continue;
            };
            for alias in &food.aliases {
                if !alias.name.trim().is_empty()
                    && normalise(&entry.name) != normalise(&alias.name)
                    && !entry
                        .aliases
                        .iter()
                        .any(|seen| normalise(seen) == normalise(&alias.name))
                {
                    entry.aliases.push(alias.name.trim().to_owned());
                }
            }
            FoodResolution::New(index)
        };
        plan.foods.insert(food.id.clone(), resolution);
    }
    plan
}

/// Maps recipe fields, ingredient lines, instructions, rating and category names.
#[must_use]
pub fn map_recipe(recipe: &Recipe, foods: &FoodPlan, units: &[MealieUnit]) -> MappedRecipe {
    let lines = recipe
        .ingredients
        .iter()
        .map(|line| map_line(line, foods, units))
        .collect();
    let steps = recipe
        .instructions
        .iter()
        .filter_map(|step| {
            let text = step.text.trim();
            if text.is_empty() {
                return None;
            }
            let text = step
                .title
                .as_deref()
                .filter(|title| !title.trim().is_empty())
                .map_or_else(
                    || text.to_owned(),
                    |title| format!("{}: {text}", title.trim()),
                );
            Some(MappedStep { text })
        })
        .collect();
    let mut category_names = Vec::new();
    let mut seen = Vec::new();
    for name in recipe
        .categories
        .iter()
        .map(|item| &item.name)
        .chain(recipe.tags.iter().map(|item| &item.name))
    {
        let name = name.trim();
        let key = normalise(name);
        if !name.is_empty() && !seen.contains(&key) {
            seen.push(key);
            category_names.push(name.to_owned());
        }
    }
    MappedRecipe {
        mealie_id: recipe.id.clone(),
        title: recipe.name.clone(),
        servings: recipe
            .servings
            .and_then(valid_servings)
            .or_else(|| recipe.yield_text.as_deref().and_then(yield_servings)),
        image_id: recipe
            .image
            .clone()
            .filter(|image| !image.trim().is_empty()),
        rating: recipe.rating.and_then(map_rating),
        categories: category_names,
        steps,
        lines,
    }
}

/// Maps source categories and tags to existing or new categories.
#[must_use]
pub fn map_categories(
    categories: &[Category],
    tags: &[Tag],
    existing: &[(i64, String)],
) -> CategoryPlan {
    let mut seen = Vec::new();
    let mut result = Vec::new();
    for name in categories
        .iter()
        .map(|item| item.name.as_str())
        .chain(tags.iter().map(|item| item.name.as_str()))
    {
        let name = name.trim();
        let key = normalise(name);
        if name.is_empty() || seen.iter().any(|prior| prior == &key) {
            continue;
        }
        seen.push(key);
        let found = existing
            .iter()
            .find(|(_, candidate)| normalise(candidate) == normalise(name));
        let resolution = found.map_or_else(
            || CategoryResolution::New(name.to_owned()),
            |(id, _)| CategoryResolution::Existing(*id),
        );
        result.push((name.to_owned(), resolution));
    }
    CategoryPlan { categories: result }
}

fn map_line(line: &Ingredient, foods: &FoodPlan, units: &[MealieUnit]) -> MappedLine {
    let quantity = line.quantity.and_then(|amount| {
        line.unit
            .as_ref()
            .and_then(|reference| measure_of_reference(reference, units, amount))
    });
    let mapped_amount = quantity
        .map(crabbage_domain::quantity::Quantity::amount)
        .or_else(|| (line.unit.is_none()).then_some(line.quantity).flatten());
    let unknown_unit = line
        .unit
        .as_ref()
        .filter(|_| quantity.is_none())
        .map(|reference| unit_text(reference, units))
        .filter(|name| !name.is_empty());
    let mut note_parts = Vec::new();
    if let Some(unit_name) = unknown_unit {
        let amount = line.quantity.map(format_amount).unwrap_or_default();
        note_parts.push(format!("{amount} {unit_name}").trim().to_owned());
    }
    if let Some(note) = line.note.as_deref().filter(|note| !note.trim().is_empty()) {
        note_parts.push(note.trim().to_owned());
    }
    let note = (!note_parts.is_empty()).then(|| note_parts.join(", "));
    let original = line
        .original_text
        .clone()
        .or_else(|| line.display.clone())
        .unwrap_or_default();
    let food = line.food.as_ref().and_then(|food| foods.resolve(&food.id));
    MappedLine {
        amount: mapped_amount,
        unit: quantity.map(crabbage_domain::quantity::Quantity::unit),
        food,
        note,
        text: food.is_none().then(|| {
            line.note
                .clone()
                .filter(|note| !note.trim().is_empty())
                .unwrap_or_else(|| original.clone())
        }),
        original,
    }
}

fn measure_of_reference(
    reference: &super::UnitRef,
    units: &[MealieUnit],
    amount: f64,
) -> Option<crabbage_domain::quantity::Quantity> {
    let measure = units
        .iter()
        .find(|unit| unit.id == reference.id)
        .and_then(|unit| {
            [
                unit.abbreviation.as_str(),
                unit.name.as_str(),
                unit.plural_name.as_str(),
            ]
            .into_iter()
            .find_map(measure_of)
        })
        .or_else(|| {
            [reference.abbreviation.as_str(), reference.name.as_str()]
                .into_iter()
                .find_map(measure_of)
        })?;
    measure.quantity(amount)
}

fn unit_text<'a>(reference: &'a super::UnitRef, units: &'a [MealieUnit]) -> &'a str {
    units
        .iter()
        .find(|unit| unit.id == reference.id)
        .map_or_else(
            || {
                if reference.abbreviation.trim().is_empty() {
                    reference.name.trim()
                } else {
                    reference.abbreviation.trim()
                }
            },
            |unit| {
                if unit.plural_name.trim().is_empty() {
                    unit.name.trim()
                } else {
                    unit.plural_name.trim()
                }
            },
        )
}

fn valid_servings(value: f64) -> Option<u32> {
    (value.is_finite() && value >= 1.0 && value.fract() == 0.0)
        .then(|| format_amount(value).parse().ok())
        .flatten()
}

fn yield_servings(value: &str) -> Option<u32> {
    let start = value
        .trim()
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>();
    start.parse().ok().filter(|count| *count > 0)
}

fn map_rating(value: f64) -> Option<u8> {
    if !value.is_finite() || value <= 0.0 {
        None
    } else {
        format_amount(value.clamp(1.0, 5.0).round()).parse().ok()
    }
}

fn format_amount(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}
