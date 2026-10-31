ALTER TABLE recipes ADD COLUMN mealie_id TEXT;
CREATE UNIQUE INDEX recipes_mealie_id ON recipes(mealie_id) WHERE mealie_id IS NOT NULL;
