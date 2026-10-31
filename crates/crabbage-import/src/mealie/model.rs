use serde::{Deserialize, Deserializer};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
/// A recipe entry returned by the recipe collection.
pub struct RecipeSummary {
    #[serde(default)]
    /// The `id:` value from Mealie.
    pub id: String,
    #[serde(default)]
    /// The `slug:` value from Mealie.
    pub slug: String,
    #[serde(default)]
    /// The `name:` value from Mealie.
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
/// A food available in Mealie.
pub struct Food {
    #[serde(default)]
    /// The `id:` value from Mealie.
    pub id: String,
    #[serde(default)]
    /// The `name:` value from Mealie.
    pub name: String,
    #[serde(default, deserialize_with = "null_default")]
    /// The `aliases:` value from Mealie.
    pub aliases: Vec<FoodAlias>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
/// An alternate name for a food.
pub struct FoodAlias {
    #[serde(default)]
    /// The `name:` value from Mealie.
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
/// A unit available in Mealie.
pub struct MealieUnit {
    #[serde(default)]
    /// The `id:` value from Mealie.
    pub id: String,
    #[serde(default)]
    /// The `name:` value from Mealie.
    pub name: String,
    #[serde(default)]
    /// The `abbreviation:` value from Mealie.
    pub abbreviation: String,
    #[serde(default, rename = "pluralName")]
    /// The plural name for the unit.
    pub plural_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
/// A recipe category.
pub struct Category {
    #[serde(default)]
    /// The `id:` value from Mealie.
    pub id: String,
    #[serde(default)]
    /// The `name:` value from Mealie.
    pub name: String,
    #[serde(default)]
    /// The `slug:` value from Mealie.
    pub slug: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
/// A recipe tag.
pub struct Tag {
    #[serde(default)]
    /// The `id:` value from Mealie.
    pub id: String,
    #[serde(default)]
    /// The `name:` value from Mealie.
    pub name: String,
    #[serde(default)]
    /// The `slug:` value from Mealie.
    pub slug: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
/// A Mealie recipe with the fields needed for import.
pub struct Recipe {
    #[serde(default)]
    /// The `id:` value from Mealie.
    pub id: String,
    #[serde(default)]
    /// The `slug:` value from Mealie.
    pub slug: String,
    #[serde(default)]
    /// The `name:` value from Mealie.
    pub name: String,
    #[serde(default, rename = "recipeServings")]
    /// The `servings:` value from Mealie.
    pub servings: Option<f64>,
    #[serde(default, rename = "recipeYield")]
    /// The `yield_text:` value from Mealie.
    pub yield_text: Option<String>,
    #[serde(
        default,
        deserialize_with = "null_default",
        rename = "recipeIngredient"
    )]
    /// The `ingredients:` value from Mealie.
    pub ingredients: Vec<Ingredient>,
    #[serde(
        default,
        deserialize_with = "null_default",
        rename = "recipeInstructions"
    )]
    /// The `instructions:` value from Mealie.
    pub instructions: Vec<Instruction>,
    #[serde(default, deserialize_with = "null_default", rename = "recipeCategory")]
    /// The `categories:` value from Mealie.
    pub categories: Vec<Category>,
    #[serde(default, deserialize_with = "null_default")]
    /// The `tags:` value from Mealie.
    pub tags: Vec<Tag>,
    #[serde(default)]
    /// The `rating:` value from Mealie.
    pub rating: Option<f64>,
    #[serde(default)]
    /// The `image:` value from Mealie.
    pub image: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
/// One ingredient in a Mealie recipe.
pub struct Ingredient {
    #[serde(default)]
    /// The `quantity:` value from Mealie.
    pub quantity: Option<f64>,
    #[serde(default)]
    /// The `unit:` value from Mealie.
    pub unit: Option<UnitRef>,
    #[serde(default)]
    /// The `food:` value from Mealie.
    pub food: Option<FoodRef>,
    #[serde(default)]
    /// The `note:` value from Mealie.
    pub note: Option<String>,
    #[serde(default, rename = "display")]
    /// The `display:` value from Mealie.
    pub display: Option<String>,
    #[serde(default, rename = "originalText")]
    /// The `original_text:` value from Mealie.
    pub original_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
/// A unit reference embedded in an ingredient.
pub struct UnitRef {
    #[serde(default)]
    /// The `id:` value from Mealie.
    pub id: String,
    #[serde(default)]
    /// The `name:` value from Mealie.
    pub name: String,
    #[serde(default)]
    /// The `abbreviation:` value from Mealie.
    pub abbreviation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
/// A food reference embedded in an ingredient.
pub struct FoodRef {
    #[serde(default)]
    /// The `id:` value from Mealie.
    pub id: String,
    #[serde(default)]
    /// The `name:` value from Mealie.
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
/// An instruction in a Mealie recipe.
pub struct Instruction {
    #[serde(default)]
    /// The `text:` value from Mealie.
    pub text: String,
    #[serde(default)]
    /// The `title:` value from Mealie.
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Paginated<T> {
    /// The `items:` value from Mealie.
    pub items: Vec<T>,
    #[serde(default, rename = "total_pages")]
    /// The `total_pages:` value from Mealie.
    pub total_pages: usize,
    #[serde(default)]
    /// The `page:` value from Mealie.
    pub current: usize,
}

fn null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}
