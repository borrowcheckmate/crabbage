ALTER TABLE users ADD COLUMN active_household_id INTEGER REFERENCES households (id) ON DELETE SET NULL;
