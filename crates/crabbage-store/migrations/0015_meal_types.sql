-- The fixed meal types of a recipe. Existing recipes have none.
CREATE TABLE recipe_meal_types (
    recipe_id INTEGER NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    meal_type TEXT NOT NULL CHECK (meal_type IN ('breakfast', 'lunch', 'dinner', 'dessert', 'side', 'snack', 'drink')),
    PRIMARY KEY (recipe_id, meal_type)
) STRICT;
