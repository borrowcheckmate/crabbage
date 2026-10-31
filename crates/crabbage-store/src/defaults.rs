use serde::Deserialize;
use sqlx::sqlite::SqlitePool;

use crate::StoreError;

const DATA: &str = include_str!("../defaults/catalog.json");

#[derive(Deserialize)]
struct Catalog {
    sections: Vec<Section>,
    categories: Vec<Category>,
}

#[derive(Deserialize)]
struct Category {
    en: String,
    nl: String,
}

#[derive(Deserialize)]
struct Section {
    en: String,
    nl: String,
    ingredients: Vec<Ingredient>,
}

#[derive(Deserialize)]
struct Ingredient {
    en: Names,
    nl: Names,
}

#[derive(Deserialize)]
struct Names {
    name: String,
    aliases: Vec<String>,
}

/// Seeds the translated catalog once, leaving any existing catalog untouched.
///
/// # Errors
///
/// Returns an error when the language is unsupported, the defaults are invalid, or a database
/// operation fails.
pub async fn seed_defaults(pool: &SqlitePool, language: &str) -> Result<(), StoreError> {
    if !matches!(language, "en" | "nl") {
        return Err(StoreError::Conflict(
            "unsupported catalog language".to_owned(),
        ));
    }
    let catalog: Catalog =
        serde_json::from_str(DATA).map_err(|error| StoreError::Conflict(error.to_string()))?;
    let mut tx = crate::begin_write(pool).await?;
    let seeded: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM catalog_seed WHERE id = 1)")
            .fetch_one(&mut *tx)
            .await?;
    if seeded {
        return Ok(());
    }
    let has_catalog: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM ingredients) OR EXISTS (SELECT 1 FROM store_sections) OR EXISTS (SELECT 1 FROM categories)",
    )
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query("INSERT INTO catalog_seed (id) VALUES (1)")
        .execute(&mut *tx)
        .await?;
    if !has_catalog {
        insert_catalog(&mut tx, &catalog, language).await?;
    }
    tx.commit().await?;
    Ok(())
}

async fn insert_catalog(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    catalog: &Catalog,
    language: &str,
) -> Result<(), sqlx::Error> {
    for (position, section) in catalog.sections.iter().enumerate() {
        let section_name = match language {
            "en" => &section.en,
            _ => &section.nl,
        };
        let section_id: i64 = sqlx::query_scalar(
            "INSERT INTO store_sections (name, position) VALUES (?, ?) RETURNING id",
        )
        .bind(section_name)
        .bind(i64::try_from(position).unwrap_or(i64::MAX))
        .fetch_one(&mut **tx)
        .await?;
        for ingredient in &section.ingredients {
            insert_ingredient(tx, ingredient, language, section_id).await?;
        }
    }
    for category in &catalog.categories {
        let name = if language == "en" {
            &category.en
        } else {
            &category.nl
        };
        sqlx::query("INSERT INTO categories (name) VALUES (?)")
            .bind(name)
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}

async fn insert_ingredient(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ingredient: &Ingredient,
    language: &str,
    section_id: i64,
) -> Result<(), sqlx::Error> {
    let names = match language {
        "en" => &ingredient.en,
        _ => &ingredient.nl,
    };
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO ingredients (name, store_section_id, pending_review) VALUES (?, ?, 0) RETURNING id",
    )
    .bind(&names.name)
    .bind(section_id)
    .fetch_one(&mut **tx)
    .await?;
    for alias in &names.aliases {
        sqlx::query("INSERT INTO ingredient_aliases (alias, ingredient_id) VALUES (?, ?)")
            .bind(alias)
            .bind(id)
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn translations_have_unique_names_and_aliases_and_valid_sections() {
        let catalog: Catalog = serde_json::from_str(DATA).unwrap();
        for language in ["en", "nl"] {
            let mut names = HashSet::new();
            let mut aliases = HashSet::new();
            let mut section_names = HashSet::new();
            for section in &catalog.sections {
                assert!(section_names.insert(translated_section(section, language).to_lowercase()));
                for ingredient in &section.ingredients {
                    let entry = translated(ingredient, language);
                    assert!(names.insert(entry.name.to_lowercase()), "{}", entry.name);
                    for alias in &entry.aliases {
                        assert!(aliases.insert(alias.to_lowercase()), "{alias}");
                    }
                }
            }
            for alias in aliases {
                assert!(!names.contains(&alias), "{alias}");
            }
        }
    }

    fn translated<'a>(section: &'a Ingredient, language: &str) -> &'a Names {
        if language == "en" {
            &section.en
        } else {
            &section.nl
        }
    }

    fn translated_section<'a>(section: &'a Section, language: &str) -> &'a str {
        if language == "en" {
            &section.en
        } else {
            &section.nl
        }
    }
}
