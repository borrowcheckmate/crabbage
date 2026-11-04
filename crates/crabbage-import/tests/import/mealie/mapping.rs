use crabbage_domain::quantity::Unit;
use crabbage_import::matching::Candidate;
use crabbage_import::mealie::{
    Category, CategoryResolution, Food, FoodResolution, MealieUnit, Recipe, Tag, map_categories,
    map_foods, map_recipe,
};

#[expect(clippy::unwrap_used, reason = "the checked-in fixture JSON is valid")]
fn foods() -> Vec<Food> {
    serde_json::from_str(
        r#"[
          {"id":"flour","name":"Flour","aliases":[{"name":"plain flour"}]},
          {"id":"chickpea","name":"Chickpea flour","aliases":[{"name":"gram flour"}]},
          {"id":"flour-duplicate","name":" flour ","aliases":[{"name":"besan"}]}
        ]"#,
    )
    .unwrap()
}

#[test]
fn foods_match_existing_names_and_aliases_and_deduplicate_new_ingredients() {
    let aliases = vec!["plain flour".to_owned()];
    let existing = [
        Candidate {
            id: 4,
            name: "Wheat Flour",
            aliases: &aliases,
        },
        Candidate {
            id: 8,
            name: "Gram flour",
            aliases: &[],
        },
    ];
    let plan = map_foods(&foods(), &existing);
    assert_eq!(plan.resolve("flour"), Some(FoodResolution::Existing(4)));
    assert_eq!(plan.resolve("chickpea"), Some(FoodResolution::Existing(8)));
    assert_eq!(
        plan.resolve("flour-duplicate"),
        Some(FoodResolution::New(0))
    );
    assert_eq!(plan.new_ingredients[0].name, " flour ");
    assert_eq!(plan.new_ingredients[0].aliases, ["besan"]);
}

#[test]
fn recipe_mapping_converts_units_and_carries_recipe_fields() {
    let mut value: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/mealie/recipe.json")).unwrap();
    value["recipeServings"] = serde_json::Value::Null;
    value["recipeYield"] = "6 small cakes".into();
    value["recipeIngredient"][0]["unit"] =
        serde_json::json!({"id":"cup","name":"cup","abbreviation":"c"});
    value["recipeIngredient"][0]["quantity"] = 2.into();
    let recipe: Recipe = serde_json::from_value(value).unwrap();
    let plan = map_foods(&[], &[]);
    let units = [unit("cup", "cup", "c", "cups")];
    let mapped = map_recipe(&recipe, &plan, &units);
    assert_eq!(mapped.mealie_id, "recipe-42");
    assert_eq!(mapped.servings, Some(6));
    assert_eq!(mapped.image_id.as_deref(), Some("citrus-cakes.webp"));
    assert_eq!(mapped.rating, Some(5));
    assert_eq!(mapped.categories, ["Baking", "Citrus"]);
    assert_eq!(mapped.steps[0].text, "Mix: Stir the ingredients.");
    assert_eq!(mapped.lines[0].amount, Some(480.0));
    assert_eq!(mapped.lines[0].unit, Some(Unit::Millilitre));
    assert_eq!(mapped.lines[0].original, "200 g flour, sifted");
}

#[test]
fn recipe_mapping_keeps_unknown_unit_in_note_and_maps_other_metric_units() {
    let value = serde_json::json!({
        "id":"r", "name":"Soup", "rating":0,
        "recipeIngredient":[
          {"quantity":2,"unit":{"id":"handful","name":"handful","abbreviation":"handful"},"note":"chopped","originalText":"2 handfuls beans"},
          {"quantity":1,"unit":{"id":"tbsp","name":"tablespoon","abbreviation":"tbsp"},"originalText":"1 tbsp oil"},
          {"quantity":12,"unit":{"id":"oz","name":"ounce","abbreviation":"oz"},"originalText":"12 oz tomatoes"},
          {"quantity":250,"unit":{"id":"g","name":"gram","abbreviation":"g"},"originalText":"250 g flour"},
          {"unit":{"id":"pinch","name":"pinch","abbreviation":"pinch"},"note":"salt","originalText":"pinch of salt"}
        ],
        "recipeInstructions":[{"text":"  ","title":"Ignored"},{"text":"Cook"}]
    });
    let recipe: Recipe = serde_json::from_value(value).unwrap();
    let units = [
        unit("handful", "handful", "handful", "handfuls"),
        unit("tbsp", "tablespoon", "tbsp", "tablespoons"),
        unit("oz", "ounce", "oz", "ounces"),
        unit("g", "gram", "g", "grams"),
    ];
    let mapped = map_recipe(&recipe, &map_foods(&[], &[]), &units);
    assert_eq!(mapped.lines[0].amount, None);
    assert_eq!(mapped.lines[0].unit, None);
    assert_eq!(mapped.lines[0].note.as_deref(), Some("2 handfuls, chopped"));
    assert_eq!(mapped.lines[1].unit, Some(Unit::Tablespoon));
    assert_eq!(mapped.lines[2].amount, Some(340.194_277_5));
    assert_eq!(mapped.lines[2].unit, Some(Unit::Gram));
    assert_eq!(mapped.lines[3].amount, Some(250.0));
    assert_eq!(mapped.lines[4].amount, None);
    assert_eq!(mapped.lines[4].note.as_deref(), Some("pinch, salt"));
    assert_eq!(mapped.rating, None);
    assert_eq!(mapped.steps.len(), 1);
}

#[test]
fn categories_and_tags_deduplicate_against_existing_names() {
    let categories = [category("Dinner"), category("  QUICK ")];
    let tags = [tag("dinner"), tag("Family")];
    let plan = map_categories(&categories, &tags, &[(12, " dinner ".to_owned())]);
    assert_eq!(
        plan.categories,
        [
            ("Dinner".to_owned(), CategoryResolution::Existing(12)),
            (
                "QUICK".to_owned(),
                CategoryResolution::New("QUICK".to_owned())
            ),
            (
                "Family".to_owned(),
                CategoryResolution::New("Family".to_owned())
            ),
        ]
    );
    assert_eq!(
        plan.resolve(" dinner"),
        Some(&CategoryResolution::Existing(12))
    );
    assert_eq!(
        plan.resolve("quick"),
        Some(&CategoryResolution::New("QUICK".to_owned()))
    );
    assert_eq!(plan.resolve("Lunch"), None);
}

fn unit(id: &str, name: &str, abbreviation: &str, plural_name: &str) -> MealieUnit {
    MealieUnit {
        id: id.to_owned(),
        name: name.to_owned(),
        abbreviation: abbreviation.to_owned(),
        plural_name: plural_name.to_owned(),
    }
}

fn category(name: &str) -> Category {
    Category {
        id: String::new(),
        name: name.to_owned(),
        slug: String::new(),
    }
}

fn tag(name: &str) -> Tag {
    Tag {
        id: String::new(),
        name: name.to_owned(),
        slug: String::new(),
    }
}
