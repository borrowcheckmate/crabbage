//! Native server-rendered navigation checks with client scripting disabled.

use chromiumoxide::cdp::browser_protocol::emulation::SetScriptExecutionDisabledParams;
use chromiumoxide::page::Page;

use super::interactions::goto_hydrated;

pub(super) async fn exercise_no_javascript(
    page: &Page,
    base_url: &str,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    let disable = SetScriptExecutionDisabledParams::builder()
        .value(true)
        .build()?;
    page.execute(disable)
        .await
        .map_err(|error| error.to_string())?;
    let result = check_native_routes(page, base_url, failures).await;
    let enable = SetScriptExecutionDisabledParams::builder()
        .value(false)
        .build()?;
    page.execute(enable)
        .await
        .map_err(|error| error.to_string())?;
    goto_hydrated(page, base_url).await?;
    result
}

async fn check_native_routes(
    page: &Page,
    base_url: &str,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    page.goto(format!("{base_url}/recipes"))
        .await
        .map_err(|error| error.to_string())?;
    let rendered: bool = page
        .evaluate("Boolean(document.querySelector('main h1')?.textContent.trim())&&Boolean(document.querySelector('main a[href]'))")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    if !rendered {
        failures.push("server-rendered recipe page failed without JavaScript".to_owned());
    }
    check_native_search(page, failures).await?;
    check_native_new_recipe_link(page, base_url, failures).await
}

async fn check_native_new_recipe_link(
    page: &Page,
    base_url: &str,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    page.goto(format!("{base_url}/recipes"))
        .await
        .map_err(|error| error.to_string())?;
    if !wait_for(
        page,
        "document.querySelector(\"main a[href*='/recipes/new']\")",
    )
    .await?
    {
        failures.push(format!(
            "native new-recipe link was absent after server navigation: {}",
            page_diagnostic(page).await?
        ));
        return Ok(());
    }
    page.evaluate("sessionStorage.setItem('__crabbage_native_nav_time_origin', String(performance.timeOrigin))")
        .await
        .map_err(|error| error.to_string())?;
    let link = page
        .find_element("main a[href*='/recipes/new']")
        .await
        .map_err(|error| format!("native new-recipe link: {error}"))?;
    let click_point = link
        .clickable_point()
        .await
        .map_err(|error| format!("native new-recipe link coordinates: {error}"))?;
    let (navigation, click) = tokio::join!(
        wait_for_navigation(page, "native new-recipe link"),
        tokio::time::timeout(std::time::Duration::from_secs(15), page.click(click_point))
    );
    click
        .map_err(|_elapsed| "native new-recipe link mouse click timed out".to_owned())?
        .map_err(|error| format!("native new-recipe link mouse click: {error}"))?;
    navigation?;
    if !wait_for(
        page,
        "location.pathname === '/recipes/new' && document.querySelector('main h1')",
    )
    .await?
    {
        failures.push(
            "ordinary new-recipe link did not open its server-rendered page without JavaScript"
                .to_owned(),
        );
    }
    let full_document_navigation: bool = page
        .evaluate("performance.timeOrigin !== Number(sessionStorage.getItem('__crabbage_native_nav_time_origin')) && performance.getEntriesByType('navigation')[0]?.type === 'navigate'")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    if !full_document_navigation {
        failures
            .push("no-JavaScript new-recipe link did not perform a document navigation".to_owned());
    }
    Ok(())
}

async fn check_native_search(page: &Page, failures: &mut Vec<String>) -> Result<(), String> {
    if !wait_for(
        page,
        "document.querySelector('.recipe-search-form input[name=q]')",
    )
    .await?
    {
        failures.push(format!(
            "native recipe search field was absent after server navigation: {}",
            page_diagnostic(page).await?
        ));
        return Ok(());
    }
    page.evaluate("document.querySelector('.recipe-search-form input[name=q]')?.focus()")
        .await
        .map_err(|error| format!("native recipe search focus: {error}"))?;
    let input = page
        .find_element(".recipe-search-form input[name=q]")
        .await
        .map_err(|error| format!("native recipe search field after focus: {error}"))?;
    tokio::time::timeout(std::time::Duration::from_secs(15), input.type_str("no-js"))
        .await
        .map_err(|_elapsed| "native recipe search keyboard input timed out".to_owned())?
        .map_err(|error| format!("native recipe search input: {error}"))?;
    let (navigation, submit) = tokio::join!(
        wait_for_navigation(page, "native recipe search"),
        tokio::time::timeout(std::time::Duration::from_secs(15), input.press_key("Enter"))
    );
    submit
        .map_err(|_elapsed| "native recipe search Enter key timed out".to_owned())?
        .map_err(|error| format!("native recipe search Enter key: {error}"))?;
    navigation?;
    if !wait_for(page, "location.pathname === '/recipes' && location.search.includes('q=') && document.querySelector('.recipe-search-form')").await? {
        failures.push("native recipe search form did not submit without JavaScript".to_owned());
    }
    Ok(())
}

async fn wait_for_navigation(page: &Page, operation: &str) -> Result<(), String> {
    tokio::time::timeout(
        std::time::Duration::from_secs(15),
        page.wait_for_navigation(),
    )
    .await
    .map_err(|_error| format!("{operation} document navigation timed out"))?
    .map(|_| ())
    .map_err(|error| format!("{operation} document navigation failed: {error}"))
}

async fn page_diagnostic(page: &Page) -> Result<String, String> {
    page.evaluate("JSON.stringify({url:location.href,title:document.title,heading:document.querySelector('main h1')?.textContent,main:document.querySelector('main')?.innerText?.slice(0,240),forms:[...document.forms].map(form=>form.outerHTML.slice(0,160)),links:[...document.querySelectorAll('main a[href]')].map(link=>link.getAttribute('href'))})")
        .await
        .map_err(|error| format!("could not inspect server-rendered page: {error}"))?
        .into_value()
        .map_err(|error| error.to_string())
}

async fn wait_for(page: &Page, expression: &str) -> Result<bool, String> {
    for _ in 0..160 {
        if let Ok(value) = page.evaluate(format!("Boolean({expression})")).await
            && let Ok(ready) = value.into_value::<bool>()
            && ready
        {
            return Ok(true);
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    Ok(false)
}
