-- What a household avoids: allergen and dietary warning codes. Households start with none.
CREATE TABLE household_avoid (
    household_id INTEGER NOT NULL REFERENCES households(id) ON DELETE CASCADE,
    code TEXT NOT NULL CHECK (code IN (
        'gluten', 'crustaceans', 'eggs', 'fish', 'peanuts', 'soy', 'milk', 'tree_nuts', 'celery',
        'mustard', 'sesame', 'sulphites', 'lupin', 'molluscs', 'meat',
        'raw_egg', 'raw_fish', 'rare_meat', 'unpasteurised_cheese', 'liver', 'alcohol'
    )),
    PRIMARY KEY (household_id, code)
) STRICT;
