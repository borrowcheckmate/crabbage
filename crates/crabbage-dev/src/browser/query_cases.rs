use chromiumoxide::page::Page;

use super::super::interactions::goto_hydrated;
use super::{restore_fetch, wait_for};

struct QueryCase {
    name: &'static str,
    path: &'static str,
    action: &'static str,
    destination: &'static str,
}

const CASES: &[QueryCase] = &[
    QueryCase {
        name: "recipe category",
        path: "/recipes",
        action: "document.querySelector('.filters a[href*=category]').click()",
        destination: "location.search.includes('category=')",
    },
    QueryCase {
        name: "planning week",
        path: "/plan",
        action: "document.querySelector('a[href^=\"/plan?week=\"]').click()",
        destination: "location.search.includes('week=')",
    },
    QueryCase {
        name: "ingredient confirmation",
        path: "/admin/ingredients/1",
        action: "document.querySelector('a[href*=\"confirm=delete\"]').click()",
        destination: "location.search.includes('confirm=delete')",
    },
    QueryCase {
        name: "recipe search",
        path: "/recipes",
        action: "const input=document.querySelector('input[name=q]');input.value='browser-query';input.form.requestSubmit()",
        destination: "location.search.includes('q=browser-query')",
    },
    QueryCase {
        name: "ingredient search",
        path: "/admin/ingredients",
        action: "const input=document.querySelector('input[name=q]');input.value='browser-query';input.form.requestSubmit()",
        destination: "location.search.includes('q=browser-query')",
    },
];

pub(super) async fn check_queries(
    page: &Page,
    base_url: &str,
    width: u32,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    for case in CASES {
        check_query(page, base_url, width, case, failures)
            .await
            .map_err(|error| format!("{}: {error}", case.name))?;
    }
    Ok(())
}

async fn check_query(
    page: &Page,
    base_url: &str,
    width: u32,
    case: &QueryCase,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    goto_hydrated(page, &format!("{base_url}{}", case.path)).await?;
    page.evaluate("window.__querySnapshot={document:document.documentElement,text:document.querySelector('main').innerText,height:document.querySelector('main').getBoundingClientRect().height};window.__realFetch=window.fetch.bind(window);window.fetch=(input,init)=>new Promise(resolve=>setTimeout(()=>resolve(window.__realFetch(input,init)),700))")
        .await.map_err(|error| error.to_string())?;
    page.evaluate(case.action)
        .await
        .map_err(|error| error.to_string())?;
    if wait_for(
        page,
        "document.querySelector('main')?.getAttribute('aria-busy') === 'true'",
    )
    .await?
    {
        check_pending(page, width, case.name, failures).await?;
    } else {
        failures.push(format!("{width}px {} did not announce loading", case.name));
    }
    let destination = format!(
        "{} && document.querySelector('main')?.getAttribute('aria-busy') === 'false' && document.querySelector('main h1')",
        case.destination
    );
    if !wait_for(page, &destination).await? {
        failures.push(format!("{width}px {} did not settle", case.name));
    }
    let retained: bool = page
        .evaluate("window.__querySnapshot?.document === document.documentElement")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    if !retained {
        failures.push(format!("{width}px {} reloaded the document", case.name));
    }
    restore_fetch(page).await
}

async fn check_pending(
    page: &Page,
    width: u32,
    name: &str,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    let retained: bool = page
        .evaluate("document.querySelector('main').innerText === window.__querySnapshot.text && Math.abs(document.querySelector('main').getBoundingClientRect().height-window.__querySnapshot.height) < 1 && Boolean(document.querySelector('.route-pending[role=status]')?.textContent.trim())")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    if !retained {
        failures.push(format!("{width}px {name} replaced content while loading"));
    }
    Ok(())
}
