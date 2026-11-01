use std::path::Path;
use std::time::Duration;

use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
use chromiumoxide::page::{Page, ScreenshotParams};
use serde_json::Value;

use super::run::wait_for_hydration;

pub(super) async fn exercise_interactions(
    page: &Page,
    base_url: &str,
    output: &Path,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    exercise_cook(page, base_url, failures).await?;
    exercise_import(page, base_url, output, failures).await?;
    exercise_shopping(page, base_url, output, failures).await?;
    exercise_plan(page, base_url, output, failures).await?;
    Ok(())
}

async fn exercise_cook(
    page: &Page,
    base_url: &str,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    let state = start_timers(page, base_url).await?;
    check_cook_start(&state, failures);
    mark_step_done(page, base_url, &state, failures).await?;
    let recipe = state
        .get("recipe")
        .and_then(Value::as_str)
        .ok_or_else(|| "timed demo recipe was not selected".to_owned())?;
    finish_and_dismiss_timers(page, base_url, recipe, failures).await?;
    check_cook_navigation(page, base_url, recipe, failures).await
}

async fn start_timers(page: &Page, base_url: &str) -> Result<Value, String> {
    let recipe = find_timed_recipe(page, base_url).await?;
    goto_hydrated(page, &format!("{base_url}/recipes/{recipe}/cook?step=1")).await?;
    let mut state: Value = page
        .evaluate("(() => { const links=[...document.querySelectorAll('.cook-nav a')].map(a=>a.getAttribute('href')); const done=document.querySelector('.cook-toggle')?.getAttribute('href'); const timer=document.querySelector('button[data-seconds]'); Date.now=()=>1000; timer?.click(); timer?.click(); return {links,done,timers:JSON.parse(sessionStorage.getItem('crabbage.timers')||'[]').length}; })()")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    if let Some(object) = state.as_object_mut() {
        object.insert("recipe".to_owned(), Value::String(recipe));
    }
    Ok(state)
}

async fn find_timed_recipe(page: &Page, base_url: &str) -> Result<String, String> {
    goto_hydrated(page, &format!("{base_url}/recipes")).await?;
    let links: Vec<String> = page
        .evaluate("[...document.querySelectorAll('.recipe-card h2 a')].map(link=>link.getAttribute('href'))")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    for link in links {
        let Some(recipe) = link
            .strip_prefix("/recipes/")
            .and_then(|path| path.split('/').next())
            .and_then(|id| id.split('?').next())
        else {
            continue;
        };
        goto_hydrated(page, &format!("{base_url}/recipes/{recipe}/cook?step=1")).await?;
        let has_timer: bool = page
            .evaluate("Boolean(document.querySelector('button.timer-start[data-seconds]'))")
            .await
            .map_err(|error| error.to_string())?
            .into_value()
            .map_err(|error| error.to_string())?;
        if has_timer {
            return Ok(recipe.to_owned());
        }
    }
    Err("demo recipes have no step with a named or stored duration".to_owned())
}

async fn goto_hydrated(page: &Page, url: &str) -> Result<(), String> {
    page.goto(url).await.map_err(|error| error.to_string())?;
    wait_for_hydration(page).await
}

fn check_cook_start(state: &Value, failures: &mut Vec<String>) {
    let timer_count = state.get("timers").and_then(Value::as_u64);
    let done_link = state
        .get("done")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let next_link_exists = state
        .get("links")
        .and_then(Value::as_array)
        .is_some_and(|links| {
            links
                .iter()
                .any(|link| link.as_str().is_some_and(|href| href.contains("step=2")))
        });
    if timer_count != Some(2) || !done_link.contains("done=1") || !next_link_exists {
        failures.push(format!(
            "cook next, mark-done or timer start failed: {state}"
        ));
    }
}

async fn mark_step_done(
    page: &Page,
    base_url: &str,
    state: &Value,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    let done_link = state
        .get("done")
        .and_then(Value::as_str)
        .unwrap_or_default();
    page.goto(format!("{base_url}{done_link}"))
        .await
        .map_err(|error| error.to_string())?;
    wait_for_hydration(page).await?;
    let marked_done: bool = page
        .evaluate("document.querySelector('.cook-toggle')?.getAttribute('aria-pressed') === 'true'")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    if !marked_done {
        failures.push("cook mark-done action did not update aria-pressed".to_owned());
    }
    Ok(())
}

async fn finish_and_dismiss_timers(
    page: &Page,
    base_url: &str,
    recipe: &str,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    goto_hydrated(page, &format!("{base_url}/recipes/{recipe}/cook?step=1")).await?;
    let tray: String = page.evaluate("(() => { const tray=document.querySelector('.timer-tray'); tray?.querySelector('summary')?.click(); const timers=JSON.parse(sessionStorage.getItem('crabbage.timers')||'[]'); Date.now=()=>Math.max(...timers.map(timer=>timer.ends_at_ms))+1000; return tray?.innerText||''; })()")
        .await.map_err(|error| error.to_string())?.into_value().map_err(|error| error.to_string())?;
    tokio::time::sleep(Duration::from_millis(1100)).await;
    let finished: u64 = page
        .evaluate("document.querySelectorAll('.timer-row.finished').length")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    if tray.is_empty() || finished != 2 {
        failures.push(format!(
            "timer tray finish lifecycle failed: {finished} finished"
        ));
    }
    page.evaluate("document.querySelector('.timer-row.finished button')?.click()")
        .await
        .map_err(|error| error.to_string())?;
    let remaining: u64 = page
        .evaluate("JSON.parse(sessionStorage.getItem('crabbage.timers')||'[]').length")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    if remaining != 1 {
        failures.push("finished timer could not be dismissed".to_owned());
    }
    Ok(())
}

async fn check_cook_navigation(
    page: &Page,
    base_url: &str,
    recipe: &str,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    goto_hydrated(page, &format!("{base_url}/recipes/{recipe}/cook?step=2")).await?;
    let reduced_motion = reduced_motion_respected(page).await?;
    if !reduced_motion {
        failures.push("reduced-motion preference was not respected".to_owned());
    }
    let previous: String = page
        .evaluate("document.querySelector('.cook-nav a')?.getAttribute('href')")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    if !previous.contains("step=1") {
        failures.push("cook previous-step navigation failed".to_owned());
    }
    Ok(())
}

async fn reduced_motion_respected(page: &Page) -> Result<bool, String> {
    page.execute(
        SetEmulatedMediaParams::builder()
            .features(vec![MediaFeature::new("prefers-reduced-motion", "reduce")])
            .build(),
    )
    .await
    .map_err(|error| error.to_string())?;
    page.evaluate("(() => { const reduced=matchMedia('(prefers-reduced-motion: reduce)').matches; const durations=[...document.querySelectorAll('*')].flatMap(element=>{const style=getComputedStyle(element);return [style.animationDuration,style.transitionDuration].flatMap(value=>value.split(',').map(duration=>parseFloat(duration)))}); return reduced && durations.every(seconds=>seconds <= .00001); })()")
        .await.map_err(|error| error.to_string())?.into_value().map_err(|error| error.to_string())
}

async fn exercise_import(
    page: &Page,
    base_url: &str,
    output: &Path,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    goto_hydrated(page, &format!("{base_url}/recipes/import?source=website")).await?;
    let website_tab: bool = page
        .evaluate(
            "document.querySelector('a[href*=website]')?.getAttribute('aria-current') === 'page'",
        )
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    if !website_tab {
        failures.push("website import tab did not become active".to_owned());
    }
    goto_hydrated(page, &format!("{base_url}/recipes/import?source=photos")).await?;
    let photo_tab: bool = page
        .evaluate(
            "document.querySelector('a[href*=photos]')?.getAttribute('aria-current') === 'page'",
        )
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    if !photo_tab {
        failures.push("photo import tab did not become active".to_owned());
    }
    save_interaction_screenshot(page, output, "recipes-import").await
}

async fn exercise_shopping(
    page: &Page,
    base_url: &str,
    output: &Path,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    goto_hydrated(page, &format!("{base_url}/shopping")).await?;
    tokio::time::sleep(Duration::from_millis(300)).await;
    let state: Value = page
        .evaluate("(() => { const item=[...document.querySelectorAll('button.shopping-item')].find(button=>button.getAttribute('aria-pressed')==='false') || document.querySelector('button.shopping-item'); const form=item?.closest('form'); const ids=form?.querySelector('input[name=ids]')?.value; const before=item?.getAttribute('aria-pressed'); item?.click(); return {ids,before}; })()")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    let ids = state.get("ids").and_then(Value::as_str).unwrap_or_default();
    let before = state
        .get("before")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let mut changed = false;
    for _ in 0..40 {
        let pressed: Result<bool, String> = page
            .evaluate(format!("[...document.querySelectorAll('form.shopping-row input[name=ids]')].find(input=>input.value==='{}')?.closest('form')?.querySelector('button.shopping-item')?.getAttribute('aria-pressed') === 'true'", ids.replace('\\', "\\\\").replace('\'', "\\'")))
            .await
            .map_err(|error| error.to_string())
            .and_then(|value| value.into_value().map_err(|error| error.to_string()));
        if let Ok(pressed) = pressed {
            changed = pressed != (before == "true");
            if changed {
                break;
            }
        }
        tokio::time::sleep(Duration::from_millis(125)).await;
    }
    if !changed {
        failures.push("shopping check-off did not change pressed state".to_owned());
    }
    save_interaction_screenshot(page, output, "shopping").await
}

async fn exercise_plan(
    page: &Page,
    base_url: &str,
    output: &Path,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    goto_hydrated(page, &format!("{base_url}/plan")).await?;
    let next: String = page
        .evaluate("document.querySelector('.plan-week a:last-child')?.getAttribute('href')")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    if !next.contains("week=") {
        failures.push("plan week navigation link is missing".to_owned());
    }
    goto_hydrated(page, &format!("{base_url}{next}")).await?;
    let changed: bool = page
        .evaluate("new URLSearchParams(location.search).has('week')")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    if !changed {
        failures.push("plan week navigation did not load the selected week".to_owned());
    }
    save_interaction_screenshot(page, output, "plan").await
}

async fn save_interaction_screenshot(page: &Page, output: &Path, name: &str) -> Result<(), String> {
    page.save_screenshot(
        ScreenshotParams::builder()
            .format(CaptureScreenshotFormat::Png)
            .build(),
        output.join(format!("interaction-{name}.png")),
    )
    .await
    .map_err(|error| error.to_string())?;
    Ok(())
}
