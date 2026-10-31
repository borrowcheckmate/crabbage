CREATE TABLE planner_settings (
    household_id INTEGER PRIMARY KEY REFERENCES households (id) ON DELETE CASCADE,
    lookback_weeks INTEGER NOT NULL DEFAULT 1 CHECK (lookback_weeks >= 0),
    servings INTEGER NOT NULL DEFAULT 4 CHECK (servings > 0)
) STRICT;

CREATE TABLE planner_weekdays (
    household_id INTEGER NOT NULL REFERENCES households (id) ON DELETE CASCADE,
    weekday INTEGER NOT NULL CHECK (weekday BETWEEN 0 AND 6),
    category_id INTEGER REFERENCES categories (id) ON DELETE SET NULL,
    PRIMARY KEY (household_id, weekday)
) STRICT;

CREATE TABLE plan_entries (
    id INTEGER PRIMARY KEY,
    household_id INTEGER NOT NULL REFERENCES households (id) ON DELETE CASCADE,
    date TEXT NOT NULL CHECK (
        length(date) = 10
        AND date GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]'
        AND date(date) = date
    ),
    recipe_id INTEGER REFERENCES recipes (id) ON DELETE SET NULL,
    servings INTEGER NOT NULL CHECK (servings > 0),
    UNIQUE (household_id, date)
) STRICT;

CREATE INDEX plan_entries_household_date ON plan_entries (household_id, date);
