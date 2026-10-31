CREATE TABLE shopping_items (
    id INTEGER PRIMARY KEY,
    household_id INTEGER NOT NULL REFERENCES households (id) ON DELETE CASCADE,
    ingredient_id INTEGER REFERENCES ingredients (id) ON DELETE CASCADE,
    text TEXT,
    amount REAL,
    unit TEXT,
    checked INTEGER NOT NULL DEFAULT 0,
    recipe_id INTEGER REFERENCES recipes (id) ON DELETE SET NULL,
    CHECK ((ingredient_id IS NULL) <> (text IS NULL))
) STRICT;
CREATE INDEX shopping_items_household ON shopping_items (household_id);
