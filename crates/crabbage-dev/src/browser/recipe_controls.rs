use chromiumoxide::page::Page;

use super::super::interactions::goto_hydrated;
use super::{restore_fetch, set_viewport, wait_for};

pub(super) async fn check_controls(
    page: &Page,
    base_url: &str,
    width: u32,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    set_viewport(page, width, 450).await?;
    for (path, selectors) in [
        (
            "/recipes/1",
            &[
                ".stepper a[href*=\"servings=5\"]",
                ".stepper a[href*=\"servings=4\"]",
                ".stepper a[href*=\"servings=5\"]",
                ".recipe-stepper-reset",
            ][..],
        ),
        (
            "/recipes/1/cook?step=1",
            &[
                ".cook-nav a[href*=\"step=2\"]",
                ".cook-nav a[href*=\"step=1\"]",
                ".cook-dot[href*=\"step=2\"]",
                ".cook-toggle",
            ][..],
        ),
    ] {
        goto_hydrated(page, &format!("{base_url}{path}")).await?;
        page.evaluate("window.__realFetch=window.fetch.bind(window);window.__controlFetches=0;window.fetch=(...args)=>{window.__controlFetches++;return window.__realFetch(...args)};window.__controlDocument=document.documentElement")
            .await.map_err(|error| error.to_string())?;
        for selector in selectors {
            check_control(page, selector, width, failures).await?;
        }
        check_history(page, width, failures).await?;
        restore_fetch(page).await?;
    }
    set_viewport(page, width, 900).await
}

async fn check_control(
    page: &Page,
    selector: &str,
    width: u32,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    let setup = format!(
        "(() => {{const link=document.querySelector({selector:?});link.scrollIntoView({{block:'center'}});link.focus({{preventScroll:true}});window.__controlSnapshot={{y:scrollY,from:location.href,target:link.href,text:document.querySelector('main').innerText,anchor:link.matches('.cook-toggle')?link.getBoundingClientRect().top:null}}}})()"
    );
    page.evaluate(setup)
        .await
        .map_err(|error| error.to_string())?;
    let scrolled: bool = page
        .evaluate("window.__controlSnapshot.y > 30 || document.documentElement.scrollHeight <= innerHeight")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    if !scrolled {
        failures.push(format!("{width}px {selector} scroll precondition failed"));
    }
    let control = page
        .find_element(selector)
        .await
        .map_err(|error| error.to_string())?;
    control.click().await.map_err(|error| error.to_string())?;
    if !wait_for(page, "location.href === window.__controlSnapshot.target && document.querySelector('main').innerText !== window.__controlSnapshot.text && document.querySelector('main').getAttribute('aria-busy') === 'false'").await? {
        failures.push(format!("{width}px {selector} did not update the recipe"));
    }
    page.evaluate(
        "new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)))",
    )
    .await
    .map_err(|error| error.to_string())?;
    let state: serde_json::Value = page
        .evaluate("({scroll:scrollY,saved:Math.min(window.__controlSnapshot.y,Math.max(0,document.documentElement.scrollHeight-innerHeight)),anchor:window.__controlSnapshot.anchor===null?null:document.querySelector('.cook-toggle').getBoundingClientRect().top,savedAnchor:window.__controlSnapshot.anchor,fetches:window.__controlFetches,document:document.documentElement===window.__controlDocument})")
        .await.map_err(|error| error.to_string())?
        .into_value().map_err(|error| error.to_string())?;
    let stable_scroll = [("scroll", "saved"), ("anchor", "savedAnchor")]
        .into_iter()
        .any(|(current, saved)| {
            state
                .get(current)
                .and_then(serde_json::Value::as_f64)
                .zip(state.get(saved).and_then(serde_json::Value::as_f64))
                .is_some_and(|(current, saved)| (current - saved).abs() < 3.0)
        });
    if !stable_scroll
        || state.get("fetches").and_then(serde_json::Value::as_u64) != Some(0)
        || state.get("document").and_then(serde_json::Value::as_bool) != Some(true)
    {
        failures.push(format!(
            "{width}px {selector} moved or reloaded the recipe: {state}"
        ));
    }
    Ok(())
}

async fn check_history(page: &Page, width: u32, failures: &mut Vec<String>) -> Result<(), String> {
    page.evaluate("window.__controlForward={url:location.href,y:scrollY,text:document.querySelector('main').innerText};history.back()")
        .await.map_err(|error| error.to_string())?;
    if !wait_for(page, "location.href===window.__controlSnapshot.from && document.querySelector('main').innerText===window.__controlSnapshot.text && Math.abs(scrollY-window.__controlSnapshot.y)<3").await? {
        failures.push(format!("{width}px Back did not restore the recipe control state and scroll"));
    }
    page.evaluate("history.forward()")
        .await
        .map_err(|error| error.to_string())?;
    if !wait_for(page, "location.href===window.__controlForward.url && document.querySelector('main').innerText===window.__controlForward.text && Math.abs(scrollY-window.__controlForward.y)<3 && window.__controlFetches===0 && document.documentElement===window.__controlDocument").await? {
        failures.push(format!("{width}px Forward did not restore the recipe without refetching or scrolling"));
    }
    Ok(())
}
