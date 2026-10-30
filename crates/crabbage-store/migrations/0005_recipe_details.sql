CREATE TABLE recipe_editors (
    recipe_id INTEGER NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    PRIMARY KEY (recipe_id, user_id)
) STRICT;

CREATE TABLE steps (
    id INTEGER PRIMARY KEY,
    recipe_id INTEGER NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    text TEXT NOT NULL,
    duration_seconds INTEGER CHECK (duration_seconds IS NULL OR duration_seconds > 0)
) STRICT;

CREATE TABLE step_lines (
    step_id INTEGER NOT NULL REFERENCES steps(id) ON DELETE CASCADE,
    line_id INTEGER NOT NULL REFERENCES recipe_lines(id) ON DELETE CASCADE,
    PRIMARY KEY (step_id, line_id)
) STRICT;

CREATE TABLE ratings (
    recipe_id INTEGER NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    stars INTEGER NOT NULL CHECK (stars BETWEEN 1 AND 5),
    PRIMARY KEY (recipe_id, user_id)
) STRICT;

CREATE TABLE favourites (
    recipe_id INTEGER NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    PRIMARY KEY (recipe_id, user_id)
) STRICT;
