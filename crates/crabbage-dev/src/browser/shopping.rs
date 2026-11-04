use std::path::Path;

use chromiumoxide::page::Page;
use serde_json::Value;

use super::interactions::{goto_hydrated, save_interaction_screenshot};
use super::navigation::{set_viewport, wait_for};

const SAME_DOCUMENT: &str = "document.documentElement===window.__shoppingDocument";

pub(super) async fn exercise_shopping(
    page: &Page,
    base_url: &str,
    output: &Path,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    set_viewport(page, 390, 600).await?;
    goto_hydrated(page, &format!("{base_url}/shopping")).await?;
    if wait_for(page, "document.querySelector('button.shopping-item')").await? {
        page.evaluate("window.__shoppingDocument=document.documentElement")
            .await
            .map_err(|error| error.to_string())?;
        check_item_in_place(page, failures).await?;
        add_item_in_place(page, failures).await?;
        save_interaction_screenshot(page, output, "shopping").await?;
    } else {
        failures.push("shopping list did not show its items".to_owned());
    }
    set_viewport(page, 390, 900).await
}

async fn check_item_in_place(page: &Page, failures: &mut Vec<String>) -> Result<(), String> {
    let target: Value = page
        .evaluate("(() => { const items=[...document.querySelectorAll('button.shopping-item[aria-pressed=false]')]; const item=items[Math.floor(items.length/2)]; if (!item) return null; item.scrollIntoView({block:'center'}); return {ids:item.closest('form').querySelector('input[name=ids]').value,y:scrollY}; })()")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    let (Some(ids), Some(before)) = (
        target.get("ids").and_then(Value::as_str),
        target.get("y").and_then(Value::as_f64),
    ) else {
        failures.push("shopping list has no unchecked item to check".to_owned());
        return Ok(());
    };
    if before < 30.0 {
        failures.push(format!("shopping scroll precondition failed: {target}"));
    }
    let ids = serde_json::to_string(ids).map_err(|error| error.to_string())?;
    click_row(page, &ids).await?;
    if !wait_for(
        page,
        &format!("{SAME_DOCUMENT} && {}==='true'", pressed(&ids)),
    )
    .await?
    {
        failures
            .push("checking a shopping item reloaded the page or did not check it once".to_owned());
        return Ok(());
    }
    page.evaluate(
        "new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)))",
    )
    .await
    .map_err(|error| error.to_string())?;
    let after: f64 = page
        .evaluate("scrollY")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    if (after - before).abs() >= 3.0 {
        failures.push(format!(
            "checking a shopping item moved the page from {before} to {after}"
        ));
    }
    click_row(page, &ids).await?;
    if !wait_for(
        page,
        &format!("{SAME_DOCUMENT} && {}==='false'", pressed(&ids)),
    )
    .await?
    {
        failures.push(
            "unchecking a shopping item reloaded the page or did not uncheck it once".to_owned(),
        );
    }
    Ok(())
}

fn rows(ids: &str) -> String {
    format!(
        "[...document.querySelectorAll('form.shopping-row')].filter(row=>row.querySelector('input[name=ids]').value==={ids}).map(row=>row.querySelector('button.shopping-item'))"
    )
}

fn pressed(ids: &str) -> String {
    format!(
        "{}.map(button=>button.getAttribute('aria-pressed')).join()",
        rows(ids)
    )
}

async fn click_row(page: &Page, ids: &str) -> Result<(), String> {
    page.evaluate(format!("document.querySelector('[data-shopping-target]')?.removeAttribute('data-shopping-target');{}[0]?.setAttribute('data-shopping-target','true')", rows(ids)))
        .await
        .map_err(|error| error.to_string())?;
    page.find_element("[data-shopping-target]")
        .await
        .map_err(|error| error.to_string())?
        .click()
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}

async fn add_item_in_place(page: &Page, failures: &mut Vec<String>) -> Result<(), String> {
    page.evaluate("(() => { const input=document.querySelector('#shopping-text'); input.value='Browser pears'; input.form.requestSubmit(); })()")
        .await
        .map_err(|error| error.to_string())?;
    if !wait_for(page, &format!("{SAME_DOCUMENT} && document.querySelector('#shopping-text').value==='' && [...document.querySelectorAll('.shopping-name')].filter(name=>name.textContent==='Browser pears').length===1")).await? {
        failures.push("adding a shopping item reloaded the page or did not show it once".to_owned());
    }
    Ok(())
}
