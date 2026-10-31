ALTER TABLE shopping_items ADD COLUMN plan_entry_id INTEGER REFERENCES plan_entries (id) ON DELETE CASCADE;
ALTER TABLE plan_entries ADD COLUMN on_list INTEGER NOT NULL DEFAULT 0 CHECK (on_list IN (0, 1));
