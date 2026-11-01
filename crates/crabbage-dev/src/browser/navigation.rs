use std::time::Duration;

use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use chromiumoxide::page::Page;

use super::interactions::goto_hydrated;

#[path = "history_cases.rs"]
mod history_cases;

const MAX_POLLS: usize = 160;

pub(super) async fn exercise_navigation(
    page: &Page,
    base_url: &str,
    language: &str,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    for width in [1280, 390] {
        set_viewport(page, width).await?;
        exercise_slow_navigation(page, base_url, language, width, failures)
            .await
            .map_err(|error| format!("{width}px slow route: {error}"))?;
        exercise_detail_and_admin(page, width, failures)
            .await
            .map_err(|error| format!("{width}px detail/admin routes: {error}"))?;
        exercise_rapid_navigation(page, width, failures)
            .await
            .map_err(|error| format!("{width}px rapid route: {error}"))?;
        exercise_failure(page, base_url, width, failures)
            .await
            .map_err(|error| format!("{width}px route error: {error}"))?;
        exercise_history_and_hash(page, base_url, width, failures)
            .await
            .map_err(|error| format!("{width}px history/hash: {error}"))?;
    }
    super::no_javascript::exercise_no_javascript(page, base_url, failures)
        .await
        .map_err(|error| format!("no-JavaScript navigation: {error}"))
}

async fn set_viewport(page: &Page, width: u32) -> Result<(), String> {
    let viewport = SetDeviceMetricsOverrideParams::builder()
        .width(width)
        .height(900)
        .device_scale_factor(1.0)
        .mobile(width <= 600)
        .build()?;
    page.execute(viewport)
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}

async fn exercise_slow_navigation(
    page: &Page,
    base_url: &str,
    language: &str,
    width: u32,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    goto_hydrated(page, base_url).await?;
    let before: String = page
        .evaluate("document.querySelector('main')?.innerText || ''")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    page.evaluate("window.__realFetch=window.fetch.bind(window);window.fetch=(input,init)=>new Promise(resolve=>setTimeout(()=>resolve(window.__realFetch(input,init)),600))")
        .await
        .map_err(|error| error.to_string())?;
    page.evaluate("window.__navSnapshot={document:document.documentElement,topbar:document.querySelector('.topbar'),primary:document.querySelector('.primary-nav'),mobile:document.querySelector('.mobile-nav'),content:document.querySelector('main').firstElementChild,entries:performance.getEntriesByType('navigation').length,height:document.querySelector('main').getBoundingClientRect().height};")
        .await
        .map_err(|error| error.to_string())?;
    click_navigation(page, width, "recipes").await?;
    if !wait_for(
        page,
        "document.querySelector('main')?.getAttribute('aria-busy') === 'true'",
    )
    .await?
    {
        failures.push(format!("{width}px route never announced pending state"));
        restore_fetch(page).await?;
        return Ok(());
    }
    for _ in 0..6 {
        let pending = read_pending_state(page).await?;
        check_pending_state(&pending, &before, language, width, failures);
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    if !wait_for(page, "location.pathname === '/recipes' && document.querySelector('main')?.getAttribute('aria-busy') === 'false' && document.querySelector('.recipe-card') && document.activeElement?.id === 'main-content'").await? {
        failures.push(format!("{width}px slow route did not finish with recipes"));
        restore_fetch(page).await?;
        return Ok(());
    }
    let focus_main: bool = page
        .evaluate("document.activeElement?.id === 'main-content'")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    if !focus_main {
        failures.push(format!(
            "{width}px route did not focus the destination main landmark"
        ));
    }
    restore_fetch(page).await?;
    Ok(())
}

async fn read_pending_state(page: &Page) -> Result<serde_json::Value, String> {
    page.evaluate("({text:document.querySelector('main')?.innerText||'',height:document.querySelector('main')?.getBoundingClientRect().height||0,beforeHeight:window.__navSnapshot.height,status:document.querySelector('.route-pending')?.innerText||'',focusMain:document.activeElement?.id==='main-content',oldContent:document.querySelector('main')?.firstElementChild===window.__navSnapshot.content,shell:document.documentElement===window.__navSnapshot.document&&document.querySelector('.topbar')===window.__navSnapshot.topbar&&document.querySelector('.primary-nav')===window.__navSnapshot.primary&&document.querySelector('.mobile-nav')===window.__navSnapshot.mobile,entries:performance.getEntriesByType('navigation').length,baseEntries:window.__navSnapshot.entries})")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())
}

fn check_pending_state(
    pending: &serde_json::Value,
    before: &str,
    language: &str,
    width: u32,
    failures: &mut Vec<String>,
) {
    let kept_content = pending.get("text").and_then(serde_json::Value::as_str) == Some(before);
    let stable_height = pending
        .get("height")
        .and_then(serde_json::Value::as_f64)
        .zip(
            pending
                .get("beforeHeight")
                .and_then(serde_json::Value::as_f64),
        )
        .is_some_and(|(height, before)| (height - before).abs() < 1.0);
    let expected_status = if language == "nl" {
        "Pagina laden…"
    } else {
        "Loading page…"
    };
    let status = pending.get("status").and_then(serde_json::Value::as_str) == Some(expected_status);
    let focus_main = pending.get("focusMain") == Some(&serde_json::Value::Bool(true));
    let old_content = pending.get("oldContent") == Some(&serde_json::Value::Bool(true));
    let shell = pending.get("shell") == Some(&serde_json::Value::Bool(true));
    let entries = pending.get("entries").and_then(serde_json::Value::as_u64)
        == pending
            .get("baseEntries")
            .and_then(serde_json::Value::as_u64);
    if !kept_content
        || !stable_height
        || !status
        || focus_main
        || !old_content
        || !shell
        || !entries
    {
        failures.push(format!(
            "{width}px slow route did not preserve content/shell: {pending}"
        ));
    }
}

async fn exercise_rapid_navigation(
    page: &Page,
    width: u32,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    page.evaluate("window.__realFetch=window.fetch.bind(window);window.fetch=(input,init)=>new Promise(resolve=>setTimeout(()=>resolve(window.__realFetch(input,init)),450))")
        .await
        .map_err(|error| error.to_string())?;
    click_navigation(page, width, "plan").await?;
    if !wait_for(
        page,
        "document.querySelector('main')?.getAttribute('aria-busy') === 'true'",
    )
    .await?
    {
        failures.push(format!(
            "{width}px first rapid route did not enter pending state"
        ));
        restore_fetch(page).await?;
        return Ok(());
    }
    click_navigation_without_scroll(page, width, "shopping").await?;
    if !wait_for(page, "location.pathname === '/shopping' && document.querySelector('main')?.getAttribute('aria-busy') === 'false' && document.querySelector('.shopping-add')").await? {
        failures.push(format!("{width}px consecutive clicks did not settle on Shopping"));
    }
    restore_fetch(page).await?;
    let shell: bool = page
        .evaluate("document.documentElement===window.__navSnapshot.document&&document.querySelector('.topbar')===window.__navSnapshot.topbar")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    if !shell {
        failures.push(format!(
            "{width}px consecutive navigation replaced the document shell"
        ));
    }
    Ok(())
}

async fn exercise_detail_and_admin(
    page: &Page,
    width: u32,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    click_selector(page, ".recipe-card h2 a").await?;
    if !wait_for(
        page,
        "location.pathname.startsWith('/recipes/') && document.querySelector('article.recipe-detail')",
    )
    .await?
    {
        failures.push(format!("{width}px recipe card did not open its detail route"));
        return Ok(());
    }
    if !shell_is_same(page).await? {
        failures.push(format!(
            "{width}px recipe detail replaced the persistent shell"
        ));
    }
    page.evaluate("history.back()")
        .await
        .map_err(|error| error.to_string())?;
    if !wait_for(
        page,
        "location.pathname === '/recipes' && document.querySelector('.recipe-card')",
    )
    .await?
    {
        failures.push(format!("{width}px Back did not return from recipe detail"));
        return Ok(());
    }
    click_selector(page, ".account-menu summary").await?;
    click_selector(page, ".account-panel a[href='/admin/ingredients']").await?;
    if !wait_for(
        page,
        "location.pathname === '/admin/ingredients' && document.querySelector('.admin-page h1')",
    )
    .await?
    {
        failures.push(format!(
            "{width}px account menu did not open administration"
        ));
    }
    if !shell_is_same(page).await? {
        failures.push(format!(
            "{width}px admin route replaced the persistent shell"
        ));
    }
    Ok(())
}

async fn shell_is_same(page: &Page) -> Result<bool, String> {
    page.evaluate("document.documentElement===window.__navSnapshot.document&&document.querySelector('.topbar')===window.__navSnapshot.topbar&&document.querySelector('.primary-nav')===window.__navSnapshot.primary&&document.querySelector('.mobile-nav')===window.__navSnapshot.mobile&&performance.getEntriesByType('navigation').length===window.__navSnapshot.entries")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())
}

async fn exercise_failure(
    page: &Page,
    _base_url: &str,
    width: u32,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    page.evaluate("window.__realFetch=window.fetch.bind(window);window.__failureInjected=false;window.fetch=(input,init)=>{const url=typeof input==='string'?input:input.url;if(url.includes('/api/list_recipes')){window.__failureInjected=true;return Promise.reject(new TypeError('simulated offline'))}return window.__realFetch(input,init)}")
        .await
        .map_err(|error| error.to_string())?;
    click_navigation(page, width, "recipes").await?;
    if !wait_for(page, "window.__failureInjected && document.querySelector('main .notice.error,main [role=alert]') && document.querySelector('main')?.getAttribute('aria-busy') === 'false'").await? {
        failures.push(format!("{width}px failed route did not settle into its error state"));
    }
    restore_fetch(page).await?;
    Ok(())
}

async fn exercise_history_and_hash(
    page: &Page,
    base_url: &str,
    width: u32,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    goto_hydrated(page, &format!("{base_url}/recipes")).await?;
    if !wait_for(page, "document.querySelector('.recipe-card')").await? {
        failures.push(format!("{width}px history check has no recipe cards"));
        return Ok(());
    }
    check_back_forward(page, width, failures).await?;
    history_cases::check_short_destination_back(page, base_url, width, failures).await?;
    history_cases::check_duplicate_url_entries(page, base_url, width, failures).await?;
    history_cases::check_query_history_entry(page, base_url, width, failures).await?;
    history_cases::check_skip_link(page, base_url, width, failures).await
}

async fn check_back_forward(
    page: &Page,
    width: u32,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    page.evaluate("window.scrollTo(0,700)")
        .await
        .map_err(|error| error.to_string())?;
    let saved_scroll: f64 = page
        .evaluate("scrollY")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    click_navigation_without_scroll(page, width, "shopping").await?;
    if !wait_for(
        page,
        "location.pathname === '/shopping' && document.querySelector('.shopping-add')",
    )
    .await?
    {
        failures.push(format!("{width}px Shopping route did not load before Back"));
        return Ok(());
    }
    let destination_scroll: f64 = page
        .evaluate("scrollY")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    if destination_scroll > 1.0 {
        failures.push(format!(
            "{width}px new destination did not start at the top: scrollY={destination_scroll}"
        ));
    }
    let back_loaded = navigate_back_and_check_scroll(page, width, saved_scroll, failures).await?;
    if !back_loaded {
        failures.push(format!("{width}px Back did not restore Recipes"));
    }
    check_forward(page, width, failures).await
}

async fn navigate_back_and_check_scroll(
    page: &Page,
    width: u32,
    saved_scroll: f64,
    failures: &mut Vec<String>,
) -> Result<bool, String> {
    page.evaluate("history.back()")
        .await
        .map_err(|error| error.to_string())?;
    let back_loaded = wait_for(
        page,
        "location.pathname === '/recipes' && document.querySelector('.recipe-card')",
    )
    .await?;
    if back_loaded {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let restored: f64 = page
            .evaluate("scrollY")
            .await
            .map_err(|error| error.to_string())?
            .into_value()
            .map_err(|error| error.to_string())?;
        if (restored - saved_scroll).abs() > 2.0 {
            failures.push(format!(
                "{width}px Back restored scroll {restored}, expected {saved_scroll}"
            ));
        }
    }
    Ok(back_loaded)
}

async fn check_forward(page: &Page, width: u32, failures: &mut Vec<String>) -> Result<(), String> {
    page.evaluate("history.forward()")
        .await
        .map_err(|error| error.to_string())?;
    if !wait_for(
        page,
        "location.pathname === '/shopping' && document.querySelector('.shopping-add')",
    )
    .await?
    {
        failures.push(format!("{width}px Forward did not restore Shopping"));
    }
    Ok(())
}

async fn click_navigation(page: &Page, width: u32, route: &str) -> Result<(), String> {
    let selector = if width <= 600 {
        format!(".mobile-nav a[href='/{route}']")
    } else {
        format!(".primary-nav a[href='/{route}']")
    };
    click_selector(page, &selector).await
}

pub(super) async fn click_navigation_without_scroll(
    page: &Page,
    width: u32,
    route: &str,
) -> Result<(), String> {
    let selector = if width <= 600 {
        format!(".mobile-nav a[href='/{route}']")
    } else {
        format!(".primary-nav a[href='/{route}']")
    };
    page.evaluate(format!("document.querySelector({selector:?})?.click()"))
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub(super) async fn click_selector(page: &Page, selector: &str) -> Result<(), String> {
    page.find_element(selector)
        .await
        .map_err(|error| format!("navigation link {selector} missing: {error}"))?
        .click()
        .await
        .map(|_| ())
        .map_err(|error| error.to_string())
}

pub(super) async fn wait_for(page: &Page, expression: &str) -> Result<bool, String> {
    for _ in 0..MAX_POLLS {
        if let Ok(value) = page.evaluate(format!("Boolean({expression})")).await
            && let Ok(found) = value.into_value::<bool>()
            && found
        {
            return Ok(true);
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    Ok(false)
}

pub(super) async fn restore_fetch(page: &Page) -> Result<(), String> {
    page.evaluate(
        "if(window.__realFetch){window.fetch=window.__realFetch;delete window.__realFetch}",
    )
    .await
    .map_err(|error| error.to_string())?;
    Ok(())
}
