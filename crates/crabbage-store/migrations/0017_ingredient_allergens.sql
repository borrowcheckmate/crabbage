-- The allergens of each catalogue ingredient, and whether the seeded catalogue got its allergens.
CREATE TABLE ingredient_allergens (
    ingredient_id INTEGER NOT NULL REFERENCES ingredients (id) ON DELETE CASCADE,
    allergen TEXT NOT NULL CHECK (allergen IN ('gluten', 'crustaceans', 'eggs', 'fish', 'peanuts', 'soy', 'milk', 'tree_nuts', 'celery', 'mustard', 'sesame', 'sulphites', 'lupin', 'molluscs', 'meat')),
    PRIMARY KEY (ingredient_id, allergen)
) STRICT;

CREATE TABLE allergen_seed (
    id INTEGER PRIMARY KEY CHECK (id = 1)
) STRICT;
