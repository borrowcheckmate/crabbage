//! Turns a recipe read from a web page into the form the user reviews, and reads the page. The
//! form is saved like a typed one, so ingredient names that match no catalogue ingredient are
//! created pending review by the store.

use crabbage_import::ai::AiClient;
use crabbage_import::lines::parse_line;
use crabbage_import::matching::Candidate;
use crabbage_import::pipeline::{Imported, complete};
use crabbage_import::web::Draft;
use crabbage_import::web::{ImportError, Limits, extract, fetch};
use crabbage_store::catalog::categories;
use crabbage_store::catalog::ingredients::{Ingredient, list};
use crabbage_store::settings;
use leptos::prelude::ServerFnError;
use sqlx::sqlite::SqlitePool;

use super::types::{ImportPreview, LineDraft, RecipeDraft, StepDraft};
use crate::access::store_failure;
use crate::config::Config;
use crate::i18n::{t, t_with};

const DEFAULT_SERVINGS: &str = "2";

/// The form for `imported`, filled the way the recipe form expects its text: amounts as numbers,
/// unit symbols, ingredient names and notes. A step's line position that is not a line of the
/// recipe is dropped.
#[must_use]
pub(crate) fn recipe_draft(imported: &Imported) -> RecipeDraft {
    let lines: Vec<LineDraft> = imported
        .lines
        .iter()
        .map(|line| LineDraft {
            amount: line
                .amount
                .map(|amount| amount.to_string())
                .unwrap_or_default(),
            unit: line
                .unit
                .map(|unit| unit.symbol().to_owned())
                .unwrap_or_default(),
            ingredient: line.name.clone(),
            note: line.note.clone().unwrap_or_default(),
        })
        .collect();
    let line_count = lines.len();
    let steps = imported
        .steps
        .iter()
        .map(|step| StepDraft {
            text: step.text.clone(),
            minutes: String::new(),
            line_positions: step
                .line_positions
                .iter()
                .copied()
                .filter(|position| *position < line_count)
                .collect(),
        })
        .collect();
    RecipeDraft {
        title: imported.title.clone(),
        servings: imported.servings.map_or_else(
            || DEFAULT_SERVINGS.to_owned(),
            |servings| servings.to_string(),
        ),
        lines,
        steps,
        categories: imported.categories.clone(),
    }
}

/// The preview of `imported`, read from `source`. The unresolved names are those of the lines that
/// match no ingredient, once each. `ai_enabled` tells whether an unresolved name could still be
/// resolved by the AI.
#[must_use]
pub(crate) fn preview(imported: &Imported, source: &str, ai_enabled: bool) -> ImportPreview {
    let mut unresolved: Vec<String> = Vec::new();
    for line in imported
        .lines
        .iter()
        .filter(|line| line.ingredient_id.is_none())
    {
        if !unresolved.contains(&line.name) {
            unresolved.push(line.name.clone());
        }
    }
    let ai_error = match (&imported.ai_error, ai_enabled) {
        (Some(error), _) => Some(t_with("import.ai_failed", &[("error", error)])),
        (None, false) if !unresolved.is_empty() => Some(t("import.ai_off").to_owned()),
        (None, _) => None,
    };
    ImportPreview {
        draft: recipe_draft(imported),
        image: imported.image.clone(),
        source: source.to_owned(),
        unresolved,
        ai_error,
        photo_preview: None,
        categories: Vec::new(),
    }
}

/// Fetches the page at `url`, reads its recipe and completes it against the catalogue, with the AI
/// when it is configured.
///
/// # Errors
///
/// Returns the translated reason when the page cannot be fetched or has no recipe, or when the
/// store fails.
pub(crate) async fn read_page(
    pool: &SqlitePool,
    config: &Config,
    url: &str,
) -> Result<ImportPreview, ServerFnError> {
    let url = url.trim();
    let limits = Limits {
        allow_private: config.import_allow_private,
        ..Limits::default()
    };
    let html = fetch(url, &limits).await.map_err(|error| failure(&error))?;
    let draft = extract(&html).map_err(|error| failure(&error))?;
    let ingredients = list(pool).await.map_err(|error| store_failure(&error))?;
    let candidates = candidates(&ingredients);
    let category_rows = categories::list(pool)
        .await
        .map_err(|error| store_failure(&error))?;
    let category_refs: Vec<(i64, &str)> = category_rows
        .iter()
        .map(|category| (category.id, category.name.as_str()))
        .collect();
    let settings = settings::get(pool)
        .await
        .map_err(|_error| ServerFnError::new(t("catalog.error.database")))?;
    let ai = ai_client(&settings);
    let imported = complete(
        &draft,
        &candidates,
        &category_refs,
        ai.as_ref(),
        config.language.code(),
    )
    .await;
    let mut preview = preview(&imported, url, settings.is_enabled());
    preview.categories = category_rows
        .into_iter()
        .map(crate::catalog::CategoryRow::from)
        .collect();
    Ok(preview)
}

pub(crate) async fn read_text(
    pool: &SqlitePool,
    config: &Config,
    text: &str,
    source: &str,
) -> Result<ImportPreview, ServerFnError> {
    let mut lines = text.lines().map(str::trim).filter(|line| !line.is_empty());
    let title = lines.next().unwrap_or_default().to_owned();
    let lines: Vec<String> = lines.map(str::to_owned).collect();
    let (ingredients_text, steps_text) = split_text_lines(&lines);
    let draft = Draft {
        title,
        servings: None,
        language: Some(config.language.code().to_owned()),
        image: None,
        lines: ingredients_text,
        steps: steps_text,
        prep_time: None,
        cook_time: None,
        total_time: None,
    };
    let ingredients = list(pool).await.map_err(|error| store_failure(&error))?;
    let candidates = candidates(&ingredients);
    let category_rows = categories::list(pool)
        .await
        .map_err(|error| store_failure(&error))?;
    let category_refs: Vec<(i64, &str)> = category_rows
        .iter()
        .map(|category| (category.id, category.name.as_str()))
        .collect();
    let settings = settings::get(pool)
        .await
        .map_err(|_error| ServerFnError::new(t("catalog.error.database")))?;
    let ai = ai_client(&settings);
    let imported = complete(
        &draft,
        &candidates,
        &category_refs,
        ai.as_ref(),
        config.language.code(),
    )
    .await;
    let mut preview = preview(&imported, source.trim(), settings.is_enabled());
    preview.categories = category_rows
        .into_iter()
        .map(crate::catalog::CategoryRow::from)
        .collect();
    Ok(preview)
}

fn split_text_lines(lines: &[String]) -> (Vec<String>, Vec<String>) {
    let (ingredients, steps): (Vec<_>, Vec<_>) = lines.iter().partition(|line| {
        let parsed = parse_line(line);
        parsed.amount.is_some() || parsed.unit.is_some()
    });
    (
        ingredients
            .into_iter()
            .map(|line| (*line).clone())
            .collect(),
        steps.into_iter().map(|line| (*line).clone()).collect(),
    )
}

fn ai_client(settings: &settings::AiSettings) -> Option<AiClient> {
    if !settings.is_enabled() {
        return None;
    }
    let base_url = url::Url::parse(&settings.base_url).ok()?;
    matches!(base_url.scheme(), "http" | "https")
        .then(|| AiClient::new(base_url, settings.api_key.clone(), settings.model.clone()))
}

/// The catalogue ingredients as candidates for matching names.
pub(super) fn candidates(ingredients: &[Ingredient]) -> Vec<Candidate<'_>> {
    ingredients
        .iter()
        .map(|ingredient| Candidate {
            id: ingredient.id,
            name: &ingredient.name,
            aliases: &ingredient.aliases,
        })
        .collect()
}

fn failure(error: &ImportError) -> ServerFnError {
    ServerFnError::new(match error {
        ImportError::NotHttp => t("import.error.not_http").to_owned(),
        ImportError::PrivateAddress => t("import.error.private").to_owned(),
        ImportError::Timeout => t("import.error.timeout").to_owned(),
        ImportError::TooLarge => t("import.error.too_large").to_owned(),
        ImportError::Status(401 | 403 | 429) => t("import.error.blocked_site").to_owned(),
        ImportError::Status(code) => t_with("import.error.status", &[("code", &code.to_string())]),
        ImportError::Network(message) => t_with("import.error.network", &[("message", message)]),
        ImportError::NoRecipe => t("import.error.no_recipe").to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use crabbage_domain::quantity::Unit;
    use crabbage_import::pipeline::{ImportedLine, ImportedStep};
    use crabbage_store::catalog::ingredients;
    use crabbage_store::recipes::create;

    use super::*;

    fn line(name: &str, amount: f64, unit: Option<Unit>, id: Option<i64>) -> ImportedLine {
        ImportedLine {
            amount: Some(amount),
            unit,
            name: name.to_owned(),
            note: Some("chopped".to_owned()),
            ingredient_id: id,
            original: name.to_owned(),
        }
    }

    fn recipe(lines: Vec<ImportedLine>, steps: Vec<ImportedStep>) -> Imported {
        Imported {
            title: "Okra soup".to_owned(),
            servings: None,
            image: Some("https://example.com/okra.png".to_owned()),
            photo: None,
            categories: Vec::new(),
            lines,
            steps,
            unmatched: Vec::new(),
            ai_error: None,
        }
    }

    fn step(positions: Vec<usize>) -> ImportedStep {
        ImportedStep {
            text: "Cook.".to_owned(),
            line_positions: positions,
        }
    }

    #[test]
    fn units_names_notes_and_amounts_fill_the_form_and_missing_servings_are_two() {
        let imported = recipe(
            vec![
                line("Okra", 200.0, Some(Unit::Gram), None),
                line("Salt", 0.5, None, Some(1)),
            ],
            vec![step(vec![0])],
        );
        let draft = recipe_draft(&imported);
        assert_eq!(draft.servings, "2");
        assert_eq!(draft.title, "Okra soup");
        assert_eq!(
            draft.lines[0],
            LineDraft {
                amount: "200".to_owned(),
                unit: "g".to_owned(),
                ingredient: "Okra".to_owned(),
                note: "chopped".to_owned(),
            }
        );
        assert_eq!(draft.lines[1].amount, "0.5");
        assert_eq!(draft.lines[1].unit, "");
        assert_eq!(draft.steps[0].line_positions, [0]);
        assert_eq!(draft.steps[0].minutes, "");
        assert_eq!(draft.categories, Vec::<i64>::new());
        let served = Imported {
            servings: Some(6),
            ..imported
        };
        assert_eq!(recipe_draft(&served).servings, "6");
    }

    #[test]
    fn step_positions_outside_the_lines_are_dropped() {
        let imported = recipe(
            vec![line("Okra", 1.0, None, None), line("Salt", 1.0, None, None)],
            vec![step(vec![0, 1, 2, 7])],
        );
        assert_eq!(recipe_draft(&imported).steps[0].line_positions, [0, 1]);
    }

    #[test]
    fn the_preview_lists_each_unresolved_name_once_and_explains_the_ai() {
        let imported = recipe(
            vec![
                line("Okra", 1.0, None, None),
                line("Salt", 1.0, None, Some(1)),
                line("Okra", 2.0, None, None),
            ],
            Vec::new(),
        );
        let off = preview(&imported, "https://example.com/soup", false);
        assert_eq!(off.unresolved, ["Okra"]);
        assert_eq!(off.source, "https://example.com/soup");
        assert_eq!(off.image.as_deref(), Some("https://example.com/okra.png"));
        assert_eq!(off.ai_error.as_deref(), Some(t("import.ai_off")));
        assert_eq!(preview(&imported, "", true).ai_error, None);
        let failed = Imported {
            ai_error: Some("the AI took too long".to_owned()),
            ..imported
        };
        let text = preview(&failed, "", true).ai_error.expect("a line");
        assert!(text.contains("the AI took too long"), "{text}");
    }

    #[test]
    fn nothing_unresolved_needs_no_ai_line() {
        let imported = recipe(vec![line("Salt", 1.0, None, Some(1))], Vec::new());
        let preview = preview(&imported, "", false);
        assert_eq!(preview.unresolved, Vec::<String>::new());
        assert_eq!(preview.ai_error, None);
    }

    #[test]
    fn each_import_error_has_its_own_message() {
        let texts: Vec<String> = [
            ImportError::NotHttp,
            ImportError::PrivateAddress,
            ImportError::Timeout,
            ImportError::TooLarge,
            ImportError::Status(404),
            ImportError::Network("refused".to_owned()),
            ImportError::NoRecipe,
        ]
        .iter()
        .map(|error| failure(error).to_string())
        .collect();
        assert!(texts[4].contains("404"), "{texts:?}");
        assert!(texts[5].contains("refused"), "{texts:?}");
        let mut unique = texts.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), texts.len(), "{texts:?}");
    }

    #[test]
    fn blocked_http_statuses_share_the_specific_message() {
        for status in [401, 403, 429] {
            assert!(
                failure(&ImportError::Status(status))
                    .to_string()
                    .contains(t("import.error.blocked_site"))
            );
        }
    }

    #[test]
    fn pasted_lines_split_english_and_dutch_ingredients_from_steps() {
        let (ingredients, steps) = split_text_lines(&[
            "2 cups flour".to_owned(),
            "Mix the flour well.".to_owned(),
            "1 tl zout".to_owned(),
            "Roer alles door elkaar.".to_owned(),
        ]);
        assert_eq!(ingredients, ["2 cups flour", "1 tl zout"]);
        assert_eq!(steps, ["Mix the flour well.", "Roer alles door elkaar."]);
    }

    #[tokio::test]
    async fn only_the_new_ingredient_is_saved_pending_review() {
        let dir = std::env::temp_dir().join(format!("crabbage-import-{}", std::process::id()));
        drop(std::fs::remove_dir_all(&dir));
        let pool = crabbage_store::open(&dir).await.expect("database opens");
        let owner = crabbage_store::users::upsert_user(&pool, "subject", "Cook")
            .await
            .expect("user");
        let salt = ingredients::create(&pool, "Salt")
            .await
            .expect("ingredient");
        let imported = recipe(
            vec![
                line("Salt", 1.0, Some(Unit::Teaspoon), Some(salt)),
                line("Okra", 200.0, Some(Unit::Gram), None),
            ],
            vec![step(vec![0, 1])],
        );
        let json = serde_json::to_string(&recipe_draft(&imported)).expect("json");
        let input = crate::recipes::draft::parse(&json).expect("the draft is valid");
        create(&pool, owner, &input, 0).await.expect("recipe saved");
        let pending: Vec<String> = ingredients::list_pending(&pool)
            .await
            .expect("pending list")
            .into_iter()
            .map(|ingredient| ingredient.name)
            .collect();
        drop(std::fs::remove_dir_all(&dir));
        assert_eq!(pending, vec!["Okra".to_owned()]);
    }
}
