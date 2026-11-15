-- The meal types a household's planner picks from. Households that saved settings plan dinner.
CREATE TABLE planner_meal_types (
    household_id INTEGER NOT NULL REFERENCES households (id) ON DELETE CASCADE,
    meal_type TEXT NOT NULL CHECK (meal_type IN ('breakfast', 'lunch', 'dinner', 'dessert', 'side', 'snack', 'drink')),
    PRIMARY KEY (household_id, meal_type)
) STRICT;

INSERT INTO planner_meal_types (household_id, meal_type)
SELECT household_id, 'dinner' FROM planner_settings;
