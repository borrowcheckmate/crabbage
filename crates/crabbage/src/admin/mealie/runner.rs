//! The Mealie migration: a background run that imports every recipe and keeps its progress.

use std::sync::Arc;

use crate::SessionUser;
use crate::state::AppState;
use axum::Extension;
use axum::Router;
use axum::extract::{Form, State};
use axum::http::StatusCode;
use axum::response::Redirect;
use axum::routing::post;
use crabbage_import::matching::Candidate;
use crabbage_import::mealie::{
    CategoryResolution, FoodResolution, MealieClient, MealieError, map_categories, map_foods,
    map_recipe,
};
use crabbage_store::recipes::{
    LineInput, RecipeInput, StepInput, create_mealie, imported_mealie_ids, rate, set_image,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use tokio::sync::Mutex;

use super::RunState;
use crate::auth::now;

/// The route that starts a migration.
pub(crate) fn routes() -> Router<AppState> {
    Router::new().route("/admin/mealie/start", post(start))
}

#[derive(Deserialize)]
struct StartForm {
    url: String,
    token: String,
}

async fn start(
    State(app): State<AppState>,
    Extension(user): Extension<SessionUser>,
    Form(form): Form<StartForm>,
) -> Result<Redirect, StatusCode> {
    let found = crabbage_store::users::find(&app.pool, user.0)
        .await
        .map_err(crate::access::internal_error)?
        .ok_or(StatusCode::FORBIDDEN)?;
    if !found.is_admin {
        return Err(StatusCode::FORBIDDEN);
    }
    {
        let mut status = app.mealie_migration.lock().await;
        if status.as_ref().is_some_and(|status| status.running) {
            return Err(StatusCode::CONFLICT);
        }
        *status = Some(RunState {
            running: true,
            ..RunState::default()
        });
    }
    let state = Arc::clone(&app.mealie_migration);
    let pool = app.pool.clone();
    let data_dir = app.config.data_dir.clone();
    tokio::spawn(async move {
        let result = migrate(&pool, user.0, &form.url, &form.token, &data_dir, &state).await;
        if let Err(error) = result {
            let mut guard = state.lock().await;
            let status = guard.get_or_insert_with(RunState::default);
            status.running = false;
            status.error = Some(error);
            drop(guard);
        }
    });
    Ok(Redirect::to("/admin/mealie"))
}

async fn migrate(
    pool: &SqlitePool,
    owner: i64,
    url: &str,
    token: &str,
    data_dir: &std::path::Path,
    state: &Arc<Mutex<Option<RunState>>>,
) -> Result<(), String> {
    let client = MealieClient::new(url, token).map_err(mealie_error)?;
    let catalog = prepare_catalog(pool, &client).await?;
    let mut imported = imported_mealie_ids(pool).await.map_err(short_error)?;
    let recipes = client.recipes().await.map_err(mealie_error)?;
    set_total(state, recipes.len()).await;
    process_recipes(
        &MigrationContext {
            pool,
            owner,
            client: &client,
            data_dir,
            state,
            catalog: &catalog,
        },
        &mut imported,
        recipes,
    )
    .await;
    let mut status = state.lock().await;
    if let Some(status) = status.as_mut() {
        status.running = false;
    }
    drop(status);
    Ok(())
}

struct Catalog {
    food_plan: crabbage_import::mealie::FoodPlan,
    ingredient_names: Vec<(i64, String)>,
    category_plan: crabbage_import::mealie::CategoryPlan,
    category_ids: Vec<(String, i64)>,
    units: Vec<crabbage_import::mealie::MealieUnit>,
}

struct MigrationContext<'a> {
    pool: &'a SqlitePool,
    owner: i64,
    client: &'a MealieClient,
    data_dir: &'a std::path::Path,
    state: &'a Arc<Mutex<Option<RunState>>>,
    catalog: &'a Catalog,
}

async fn prepare_catalog(pool: &SqlitePool, client: &MealieClient) -> Result<Catalog, String> {
    let foods = client.foods().await.map_err(mealie_error)?;
    let units = client.units().await.map_err(mealie_error)?;
    let source_categories = client.categories().await.map_err(mealie_error)?;
    let tags = client.tags().await.map_err(mealie_error)?;
    let ingredients = crabbage_store::catalog::ingredients::list(pool)
        .await
        .map_err(short_error)?;
    let candidates: Vec<Candidate<'_>> = ingredients
        .iter()
        .map(|item| Candidate {
            id: item.id,
            name: &item.name,
            aliases: &item.aliases,
        })
        .collect();
    let ingredient_names: Vec<(i64, String)> = ingredients
        .iter()
        .map(|item| (item.id, item.name.clone()))
        .collect();
    let food_plan = map_foods(&foods, &candidates);
    create_pending_ingredients(pool, &food_plan).await?;
    let categories = crabbage_store::catalog::categories::list(pool)
        .await
        .map_err(short_error)?;
    let existing: Vec<_> = categories
        .iter()
        .map(|category| (category.id, category.name.clone()))
        .collect();
    let category_plan = map_categories(&source_categories, &tags, &existing);
    let mut category_ids = Vec::new();
    for (name, resolution) in &category_plan.categories {
        let id = match resolution {
            CategoryResolution::Existing(id) => *id,
            CategoryResolution::New(name) => {
                crabbage_store::catalog::categories::create(pool, name)
                    .await
                    .map_err(short_error)?
            }
        };
        category_ids.push((name.clone(), id));
    }
    Ok(Catalog {
        food_plan,
        ingredient_names,
        category_plan,
        category_ids,
        units,
    })
}

async fn create_pending_ingredients(
    pool: &SqlitePool,
    plan: &crabbage_import::mealie::FoodPlan,
) -> Result<(), String> {
    for ingredient in &plan.new_ingredients {
        crabbage_store::catalog::ingredients::create_pending_with_aliases(
            pool,
            &ingredient.name,
            &ingredient.aliases,
        )
        .await
        .map_err(short_error)?;
    }
    Ok(())
}

async fn set_total(state: &Arc<Mutex<Option<RunState>>>, total: usize) {
    let mut status = state.lock().await;
    status.get_or_insert_with(RunState::default).total = total;
    drop(status);
}

async fn process_recipes(
    context: &MigrationContext<'_>,
    imported: &mut std::collections::HashSet<String>,
    recipes: Vec<crabbage_import::mealie::RecipeSummary>,
) {
    for summary in recipes {
        let prior = imported.contains(&summary.id);
        if prior {
            update(context.state, |status| {
                status.skipped += 1;
                status.done += 1;
            })
            .await;
            continue;
        }
        let result = import_one(context, &summary.slug).await;
        match result {
            Ok(()) => {
                imported.insert(summary.id);
                update(context.state, |status| {
                    status.imported += 1;
                    status.done += 1;
                })
                .await;
            }
            Err(reason) => {
                update(context.state, |status| {
                    status.failed.push((summary.name.clone(), reason));
                    status.done += 1;
                })
                .await;
            }
        }
    }
}

async fn import_one(context: &MigrationContext<'_>, slug: &str) -> Result<(), String> {
    let source = context.client.recipe(slug).await.map_err(mealie_error)?;
    let mapped = map_recipe(&source, &context.catalog.food_plan, &context.catalog.units);
    let rating = mapped.rating;
    let input = recipe_input(mapped, context.catalog);
    let id = create_mealie(context.pool, context.owner, &input, now(), &source.id)
        .await
        .map_err(short_error)?;
    if let Some(stars) = rating {
        rate(context.pool, id, context.owner, i64::from(stars))
            .await
            .map_err(short_error)?;
    }
    attach_image(context.pool, id, context.client, &source, context.data_dir).await;
    Ok(())
}

fn recipe_input(mapped: crabbage_import::mealie::MappedRecipe, catalog: &Catalog) -> RecipeInput {
    let lines = recipe_lines(&mapped, catalog);
    let steps = mapped
        .steps
        .iter()
        .map(|step| StepInput {
            text: step.text.clone(),
            duration_seconds: None,
            line_positions: Vec::new(),
        })
        .collect();
    let recipe_categories = recipe_categories(&mapped, catalog);
    RecipeInput {
        title: mapped.title,
        servings: i64::from(mapped.servings.unwrap_or(2)),
        lines,
        steps,
        category_ids: recipe_categories,
    }
}

fn recipe_lines(
    mapped: &crabbage_import::mealie::MappedRecipe,
    catalog: &Catalog,
) -> Vec<LineInput> {
    mapped
        .lines
        .iter()
        .map(|line| {
            let ingredient = match line.food {
                Some(FoodResolution::Existing(id)) => catalog
                    .ingredient_names
                    .iter()
                    .find(|(known, _)| *known == id)
                    .map(|(_, name)| name.clone()),
                Some(FoodResolution::New(index)) => catalog
                    .food_plan
                    .new_ingredients
                    .get(index)
                    .map(|item| item.name.clone()),
                None => None,
            };
            LineInput {
                amount: line.amount,
                unit: line.unit.map(|unit| unit.symbol().to_owned()),
                note: if ingredient.is_some() {
                    line.note.clone()
                } else {
                    line.text.clone().or_else(|| line.note.clone())
                },
                ingredient,
            }
        })
        .collect()
}

fn recipe_categories(
    mapped: &crabbage_import::mealie::MappedRecipe,
    catalog: &Catalog,
) -> Vec<i64> {
    mapped
        .categories
        .iter()
        .filter_map(|name| {
            let resolution = catalog.category_plan.resolve(name)?;
            let index = catalog
                .category_plan
                .categories
                .iter()
                .position(|(_, known)| known == resolution)?;
            catalog.category_ids.get(index).map(|(_, id)| *id)
        })
        .collect()
}

async fn attach_image(
    pool: &SqlitePool,
    id: i64,
    client: &MealieClient,
    source: &crabbage_import::mealie::Recipe,
    data_dir: &std::path::Path,
) {
    if source.image.is_some()
        && let Ok(bytes) = client.image(source).await
        && let Some(kind) = crate::images::Kind::detect(&bytes)
        && let Ok(name) = crate::images::write(data_dir, &bytes, kind).await
        && set_image(pool, id, Some(&name)).await.is_err()
    {
        drop(crate::images::remove(data_dir, &name).await);
    }
}

async fn update(state: &Arc<Mutex<Option<RunState>>>, f: impl FnOnce(&mut RunState)) {
    let mut state = state.lock().await;
    if let Some(status) = state.as_mut() {
        f(status);
    }
}

fn short_error<E: std::fmt::Display>(error: E) -> String {
    error.to_string().chars().take(120).collect()
}

fn mealie_error(error: MealieError) -> String {
    use crate::i18n::{t, t_with};
    match error {
        MealieError::InvalidUrl => t("admin.mealie.error.url").to_owned(),
        MealieError::Unauthorized => t("admin.mealie.error.unauthorized").to_owned(),
        MealieError::NotMealie | MealieError::NotFound => {
            t("admin.mealie.error.not_mealie").to_owned()
        }
        MealieError::Timeout => t("admin.mealie.error.timeout").to_owned(),
        MealieError::Network(message) => {
            t_with("admin.mealie.error.network", &[("message", &message)])
        }
        MealieError::Status(code) => {
            t_with("admin.mealie.error.status", &[("code", &code.to_string())])
        }
        MealieError::TooLarge => t("admin.mealie.error.image").to_owned(),
    }
}
