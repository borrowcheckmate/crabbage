use chromiumoxide::page::Page;

use super::super::interactions::goto_hydrated;
use super::{click_navigation_without_scroll, restore_fetch, wait_for};

pub(super) async fn check_short_destination_back(
    page: &Page,
    base_url: &str,
    width: u32,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    goto_hydrated(page, &format!("{base_url}/recipes")).await?;
    if !wait_for(page, "document.querySelector('.recipe-card')").await? {
        failures.push(format!("{width}px short-page history check has no recipes"));
        return Ok(());
    }
    page.evaluate("window.scrollTo(0,700)")
        .await
        .map_err(|error| error.to_string())?;
    let saved: f64 = page
        .evaluate("scrollY")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    page.evaluate("document.querySelector('.brand').click()")
        .await
        .map_err(|error| error.to_string())?;
    if !wait_for(page, "location.pathname === '/' && document.querySelector('main')?.getAttribute('aria-busy') === 'false'").await? {
        failures.push(format!("{width}px short Today route did not settle"));
        return Ok(());
    }
    let max_scroll: f64 = page
        .evaluate("document.documentElement.scrollHeight-window.innerHeight")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    if max_scroll + 100.0 >= saved {
        failures.push(format!(
            "{width}px Today destination was not shorter than the saved scroll: max scroll {max_scroll}, saved {saved}"
        ));
    }
    page.evaluate("window.__realFetch=window.fetch.bind(window);window.fetch=(input,init)=>new Promise(resolve=>setTimeout(()=>resolve(window.__realFetch(input,init)),650))")
        .await.map_err(|error| error.to_string())?;
    page.evaluate("history.back()")
        .await
        .map_err(|error| error.to_string())?;
    if !wait_for(page, "location.pathname === '/recipes' && document.querySelector('main')?.getAttribute('aria-busy') === 'true'").await? {
        failures.push(format!(
            "{width}px delayed Back to Recipes did not enter pending state: {}",
            history_diagnostic(page).await?
        ));
        restore_fetch(page).await?;
        return Ok(());
    }
    if !wait_for(page, "location.pathname === '/recipes' && document.querySelector('.recipe-card') && document.querySelector('main')?.getAttribute('aria-busy') === 'false'").await? {
        failures.push(format!("{width}px Back did not settle on Recipes from short destination"));
        restore_fetch(page).await?;
        return Ok(());
    }
    restore_fetch(page).await?;
    check_short_back_scroll(page, saved, width, failures).await
}

async fn check_short_back_scroll(
    page: &Page,
    saved: f64,
    width: u32,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    if !wait_for(page, &format!("Math.abs(scrollY-{saved}) < 3")).await? {
        let restored: f64 = page
            .evaluate("scrollY")
            .await
            .map_err(|error| error.to_string())?
            .into_value()
            .map_err(|error| error.to_string())?;
        failures.push(format!(
            "{width}px short destination clamped Back scroll to {restored}, expected {saved}"
        ));
    }
    Ok(())
}

pub(super) async fn check_duplicate_url_entries(
    page: &Page,
    base_url: &str,
    width: u32,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    goto_hydrated(page, &format!("{base_url}/recipes")).await?;
    if !wait_for(page, "document.querySelector('.recipe-card')").await? {
        return Ok(());
    }
    page.evaluate("window.scrollTo(0,700)")
        .await
        .map_err(|error| error.to_string())?;
    let first: f64 = page
        .evaluate("scrollY")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    page.evaluate("document.querySelector('.brand').click()")
        .await
        .map_err(|error| error.to_string())?;
    if !wait_for(page, "location.pathname === '/' && document.querySelector('main')?.getAttribute('aria-busy') === 'false'").await? {
        failures.push(format!("{width}px duplicate URL setup did not reach Today"));
        return Ok(());
    }
    click_navigation_without_scroll(page, width, "recipes").await?;
    if !wait_for(page, "location.pathname === '/recipes' && document.querySelector('.recipe-card') && document.querySelector('main')?.getAttribute('aria-busy') === 'false'").await? {
        failures.push(format!("{width}px duplicate Recipes entry did not load"));
        return Ok(());
    }
    page.evaluate("window.scrollTo(0,350)")
        .await
        .map_err(|error| error.to_string())?;
    let second: f64 = page
        .evaluate("scrollY")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    page.evaluate("history.back()")
        .await
        .map_err(|error| error.to_string())?;
    if !wait_for(page, "location.pathname === '/' && document.querySelector('main')?.getAttribute('aria-busy') === 'false'").await? {
        failures.push(format!("{width}px duplicate URL Back did not reach Today"));
        return Ok(());
    }
    page.evaluate("history.back()")
        .await
        .map_err(|error| error.to_string())?;
    if !wait_for(page, "location.pathname === '/recipes' && document.querySelector('.recipe-card') && document.querySelector('main')?.getAttribute('aria-busy') === 'false'").await? {
        failures.push(format!("{width}px duplicate URL Back did not reach first Recipes entry"));
        return Ok(());
    }
    let restored_first: f64 = page
        .evaluate("scrollY")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    if (restored_first - first).abs() > 3.0 || (second - first).abs() < 3.0 {
        failures.push(format!("{width}px duplicate URL entries did not retain distinct scroll positions: first={first}, second={second}, restored={restored_first}"));
    }
    Ok(())
}

pub(super) async fn check_query_history_entry(
    page: &Page,
    base_url: &str,
    width: u32,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    goto_hydrated(page, &format!("{base_url}/recipes")).await?;
    if !wait_for(page, "document.querySelector('.filters a[href*=category]')").await? {
        failures.push(format!(
            "{width}px recipe query test has no category filters"
        ));
        return Ok(());
    }
    check_same_route_query_history(page, width, failures).await?;
    goto_hydrated(page, &format!("{base_url}/recipes")).await?;
    page.evaluate("document.querySelector('.filters a[href*=category]').click()")
        .await
        .map_err(|error| error.to_string())?;
    if !wait_for(page, "location.pathname === '/recipes' && location.search.includes('category=') && document.querySelector('.recipe-card') && document.querySelector('main')?.getAttribute('aria-busy') === 'false'").await? {
        failures.push(format!("{width}px category query setup did not settle before path-history test"));
        return Ok(());
    }
    page.evaluate("window.scrollTo(0,500)")
        .await
        .map_err(|error| error.to_string())?;
    let saved: f64 = page
        .evaluate("scrollY")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    page.evaluate("document.querySelector('.brand[href=\\'/\\']').click()")
        .await
        .map_err(|error| error.to_string())?;
    if !wait_for(page, "location.pathname === '/' && document.querySelector('main')?.getAttribute('aria-busy') === 'false'").await? {
        failures.push(format!("{width}px category query test did not reach Today"));
        return Ok(());
    }
    page.evaluate("history.back()")
        .await
        .map_err(|error| error.to_string())?;
    if !wait_for(page, "location.pathname === '/recipes' && location.search.includes('category=') && document.querySelector('.recipe-card') && document.querySelector('main')?.getAttribute('aria-busy') === 'false'").await? {
        failures.push(format!("{width}px Back did not restore the recipe query URL"));
        return Ok(());
    }
    if !wait_for(page, &format!("Math.abs(scrollY-{saved}) < 3")).await? {
        let restored: f64 = page
            .evaluate("scrollY")
            .await
            .map_err(|error| error.to_string())?
            .into_value()
            .map_err(|error| error.to_string())?;
        failures.push(format!(
            "{width}px query-entry Back restored scroll {restored}, expected {saved}"
        ));
    }
    Ok(())
}

async fn check_same_route_query_history(
    page: &Page,
    width: u32,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    let Some((first_url, recipe_count, saved_scroll)) =
        prepare_long_query_history(page, width, failures).await?
    else {
        return Ok(());
    };
    let Some(short_url) = open_short_query(page, &first_url, recipe_count, width, failures).await?
    else {
        return Ok(());
    };
    page.evaluate("history.back()")
        .await
        .map_err(|error| error.to_string())?;
    if !wait_for(page, &format!("location.href === {first_url:?} && document.querySelectorAll('.recipe-card').length === {recipe_count}")).await? {
        failures.push(format!(
            "{width}px same-route query Back did not restore the unfiltered list: {}",
            history_diagnostic(page).await?
        ));
        restore_fetch(page).await?;
        return Ok(());
    }
    restore_fetch(page).await?;
    check_short_back_scroll(page, saved_scroll, width, failures).await?;
    page.evaluate("history.forward()")
        .await
        .map_err(|error| error.to_string())?;
    if !wait_for(page, &format!("location.href === {short_url:?} && document.querySelectorAll('.recipe-card').length < {recipe_count}")).await? {
        failures.push(format!("{width}px same-route query Forward did not restore the short category"));
    }
    Ok(())
}

async fn history_diagnostic(page: &Page) -> Result<String, String> {
    page.evaluate("JSON.stringify({href:location.href,cards:document.querySelectorAll('.recipe-card').length,scrollY,height:document.documentElement.scrollHeight,busy:document.querySelector('main')?.getAttribute('aria-busy'),state:history.state,fetches:window.__history_fetches||[]})")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())
}

async fn prepare_long_query_history(
    page: &Page,
    width: u32,
    failures: &mut Vec<String>,
) -> Result<Option<(String, usize, f64)>, String> {
    let first_url: String = page
        .evaluate("location.href")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    let recipe_count: usize = page
        .evaluate("document.querySelectorAll('.recipe-card').length")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    if recipe_count < 2 {
        failures.push(format!("{width}px query history needs a long unfiltered recipe list, found {recipe_count} cards"));
        return Ok(None);
    }
    page.evaluate("window.scrollTo(0,700)")
        .await
        .map_err(|error| error.to_string())?;
    let saved_scroll: f64 = page
        .evaluate("scrollY")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    if saved_scroll < 100.0 {
        failures.push(format!("{width}px query history precondition did not reach a nonzero long-list scroll: {saved_scroll}"));
        return Ok(None);
    }
    Ok(Some((first_url, recipe_count, saved_scroll)))
}

async fn open_short_query(
    page: &Page,
    first_url: &str,
    recipe_count: usize,
    width: u32,
    failures: &mut Vec<String>,
) -> Result<Option<String>, String> {
    page.evaluate("window.__history_fetches=[];window.__realFetch=window.fetch.bind(window);window.fetch=(input,init)=>new Promise((resolve,reject)=>setTimeout(()=>window.__realFetch(input,init).then(response=>{window.__history_fetches.push({url:String(input),status:response.status});resolve(response)},error=>{window.__history_fetches.push({url:String(input),error:String(error)});reject(error)}),2400))")
        .await
        .map_err(|error| error.to_string())?;
    page.evaluate(
        "document.querySelector('.filters a[href*=category]:not([aria-current])')?.click()",
    )
    .await
    .map_err(|error| error.to_string())?;
    if !wait_for(page, &format!("location.href !== {first_url:?} && location.pathname === '/recipes' && location.search.includes('category=') && document.querySelectorAll('.recipe-card').length > 0 && document.querySelectorAll('.recipe-card').length < {recipe_count}")).await? {
        failures.push(format!("{width}px same-route short category query did not load"));
        restore_fetch(page).await?;
        return Ok(None);
    }
    let short_max: f64 = page
        .evaluate("document.documentElement.scrollHeight-window.innerHeight")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    if short_max > 60.0 {
        failures.push(format!(
            "{width}px category destination was not short: max scroll {short_max}"
        ));
    }
    let short_url: String = page
        .evaluate("location.href")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    Ok(Some(short_url))
}

pub(super) async fn check_skip_link(
    page: &Page,
    base_url: &str,
    width: u32,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    goto_hydrated(page, &format!("{base_url}/recipes")).await?;
    page.evaluate("window.scrollTo(0,700)")
        .await
        .map_err(|error| error.to_string())?;
    let before_skip: f64 = page
        .evaluate("scrollY")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    page.evaluate("document.querySelector('a.skip-link').click()")
        .await
        .map_err(|error| error.to_string())?;
    if !wait_for(page, "location.hash === '#main-content'").await? {
        failures.push(format!("{width}px skip link did not update the hash"));
    }
    let scroll: f64 = page
        .evaluate("scrollY")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    if scroll > 100.0 {
        failures.push(format!(
            "{width}px skip link did not return to main content: scrollY={scroll}"
        ));
    }
    let focused: bool = page
        .evaluate("document.activeElement?.id === 'main-content'")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    if !focused {
        failures.push(format!(
            "{width}px skip link did not focus the main landmark"
        ));
    }
    page.evaluate("history.back()")
        .await
        .map_err(|error| error.to_string())?;
    if !wait_for(
        page,
        "location.pathname === '/recipes' && location.hash === ''",
    )
    .await?
        || !wait_for(page, &format!("Math.abs(scrollY-{before_skip}) < 3")).await?
    {
        failures.push(format!(
            "{width}px Back from the skip-link hash did not restore the prior scroll position"
        ));
    }
    Ok(())
}
