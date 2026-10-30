CREATE TABLE store_sections (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE,
    position INTEGER NOT NULL
) STRICT;

CREATE TABLE ingredients (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE,
    store_section_id INTEGER REFERENCES store_sections(id) ON DELETE SET NULL,
    pending_review INTEGER NOT NULL DEFAULT 0
) STRICT;

CREATE TABLE ingredient_aliases (
    alias TEXT PRIMARY KEY COLLATE NOCASE,
    ingredient_id INTEGER NOT NULL REFERENCES ingredients(id) ON DELETE CASCADE
) STRICT;

CREATE TABLE recipes (
    id INTEGER PRIMARY KEY,
    owner_id INTEGER NOT NULL REFERENCES users(id),
    title TEXT NOT NULL,
    servings INTEGER NOT NULL CHECK (servings > 0),
    image TEXT,
    created_at INTEGER NOT NULL
) STRICT;

CREATE TABLE recipe_lines (
    id INTEGER PRIMARY KEY,
    recipe_id INTEGER NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    amount REAL,
    unit TEXT,
    ingredient_id INTEGER REFERENCES ingredients(id) ON DELETE RESTRICT,
    note TEXT
) STRICT;
