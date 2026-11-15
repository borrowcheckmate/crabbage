-- The fixed dietary warnings of a recipe. Existing recipes have none.
CREATE TABLE recipe_warnings (
    recipe_id INTEGER NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    warning TEXT NOT NULL CHECK (warning IN ('raw_egg', 'raw_fish', 'rare_meat', 'unpasteurised_cheese', 'liver', 'alcohol')),
    PRIMARY KEY (recipe_id, warning)
) STRICT;
