use super::*;

#[tokio::test]
async fn all_lists_follow_pages_and_send_the_bearer_token() {
    let (base, mut requests) = serve(paged).await;
    let api = client(&base);
    assert_eq!(api.recipes().await.expect("recipes").len(), 3);
    assert_eq!(api.foods().await.expect("foods").len(), 3);
    assert_eq!(api.units().await.expect("units").len(), 3);
    assert_eq!(api.categories().await.expect("categories").len(), 3);
    assert_eq!(api.tags().await.expect("tags").len(), 3);
    let mut paths = Vec::new();
    for _ in 0..15 {
        let request = requests.recv().await.expect("request");
        assert!(
            request
                .to_ascii_lowercase()
                .contains("authorization: bearer mealie-test-token")
        );
        assert!(request.contains("perPage=100"));
        paths.push(
            request
                .split_whitespace()
                .nth(1)
                .unwrap_or_default()
                .to_owned(),
        );
    }
    for endpoint in [
        "/api/recipes?",
        "/api/foods?",
        "/api/units?",
        "/api/organizers/categories?",
        "/api/organizers/tags?",
    ] {
        for page in 1..=3 {
            assert!(paths.iter().any(|path| {
                path.starts_with(endpoint) && path.contains(&format!("page={page}"))
            }));
        }
    }
}

#[tokio::test]
async fn reads_full_recipe_and_downloads_image() {
    let fixture = include_str!("../fixtures/mealie/recipe.json").to_owned();
    let (base, _) = serve(move |request| {
        if request.contains("original.webp") {
            (200, "image".to_owned())
        } else {
            (200, fixture.clone())
        }
    })
    .await;
    let api = client(&base);
    let recipe = api.recipe("citrus-cakes").await.expect("recipe");
    assert_eq!(recipe.name, "Citrus Cakes");
    assert_eq!(recipe.servings, Some(4.0));
    assert_eq!(
        recipe.ingredients[0]
            .food
            .as_ref()
            .map(|food| food.name.as_str()),
        Some("flour")
    );
    assert_eq!(recipe.instructions[0].title.as_deref(), Some("Mix"));
    assert_eq!(api.image(&recipe).await.expect("image"), b"image");
}

#[tokio::test]
async fn unauthorized_status_is_clear_and_token_is_not_reported() {
    let error = error_of(401, "{}").await;
    assert_eq!(error.to_string(), "the API token was refused");
    assert!(!error.to_string().contains("mealie-test-token"));
}

#[test]
fn invalid_urls_are_rejected() {
    for url in ["ftp://mealie.example", "http://", "not a url"] {
        assert_eq!(
            MealieClient::new(url, "token").err(),
            Some(MealieError::InvalidUrl)
        );
    }
    assert_eq!(
        MealieError::InvalidUrl.to_string(),
        "the Mealie URL must be an HTTP or HTTPS URL with a host"
    );
}

#[tokio::test]
async fn html_and_not_found_are_reported_as_non_mealie() {
    let error = error_of(404, "<html>missing</html>").await;
    assert!(matches!(error, MealieError::NotMealie));
    let (base, _) = serve(|_| (200, "<html>not an API</html>".to_owned())).await;
    assert!(matches!(
        client(&base).recipes().await,
        Err(MealieError::NotMealie)
    ));
}

#[tokio::test]
async fn image_request_uses_recipe_id() {
    let (base, mut requests) = serve(|_| (200, "{}".to_owned())).await;
    let api = client(&base);
    let recipe: crabbage_import::mealie::Recipe =
        serde_json::from_str(include_str!("../fixtures/mealie/recipe.json")).expect("fixture");
    drop(api.image(&recipe).await.expect("image"));
    assert!(
        requests
            .recv()
            .await
            .expect("request")
            .contains("/api/media/recipes/recipe-42/images/original.webp")
    );
}

#[test]
fn null_recipe_collections_and_optional_fields_default_cleanly() {
    let recipe: crabbage_import::mealie::Recipe = serde_json::from_str(
        r#"{"id":"r","recipeIngredient":null,"recipeInstructions":null,"recipeCategory":null,"tags":null,"rating":null,"image":null}"#,
    ).expect("recipe");
    assert_eq!(recipe.ingredients.len(), 0);
    assert_eq!(recipe.instructions.len(), 0);
    assert_eq!(recipe.categories.len(), 0);
    assert_eq!(recipe.tags.len(), 0);
    assert_eq!(recipe.rating, None);
    assert_eq!(recipe.image, None);
}

#[tokio::test]
async fn pagination_stops_when_a_page_is_empty() {
    let (base, mut requests) = serve(|request| {
        let page = request
            .split("page=")
            .nth(1)
            .and_then(|part| part.split('&').next())
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(1);
        if page == 1 {
            (
                200,
                json!({"items":[{"id":"r","slug":"r","name":"R"}],"page":1,"total_pages":99})
                    .to_string(),
            )
        } else {
            (
                200,
                json!({"items":[],"page":page,"total_pages":99}).to_string(),
            )
        }
    })
    .await;
    let items = client(&base).recipes().await.expect("recipes");
    assert_eq!(items.len(), 1);
    assert!(
        requests
            .recv()
            .await
            .expect("first request")
            .contains("page=1")
    );
    assert!(
        requests
            .recv()
            .await
            .expect("second request")
            .contains("page=2")
    );
    assert!(requests.try_recv().is_err());
}
