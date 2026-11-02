-- Ingredient names start with a capital letter, as new ones now do. upper() only changes ASCII letters.
UPDATE ingredients SET name = upper(substr(name, 1, 1)) || substr(name, 2) WHERE substr(name, 1, 1) <> upper(substr(name, 1, 1));
