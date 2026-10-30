CREATE TABLE households (
    id INTEGER PRIMARY KEY,
    group_name TEXT NOT NULL UNIQUE
) STRICT;

CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    subject TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    is_admin INTEGER NOT NULL DEFAULT 0
) STRICT;

CREATE TABLE user_households (
    user_id INTEGER NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    household_id INTEGER NOT NULL REFERENCES households (id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, household_id)
) STRICT;
