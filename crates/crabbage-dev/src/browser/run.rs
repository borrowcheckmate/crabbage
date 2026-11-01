//! Route, accessibility and reflow checks against a running demo.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use std::time::Duration;

use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::emulation::{
    MediaFeature, SetDeviceMetricsOverrideParams, SetEmulatedMediaParams,
};
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
use chromiumoxide::page::{Page, ScreenshotParams};
use futures_util::StreamExt;
use serde_json::Value;

use super::interactions::exercise_interactions;

pub(super) const ROUTES: &[(&str, &str)] = &[
    ("today", "/"),
    ("ingredients", "/admin/ingredients"),
    ("ingredient", "/admin/ingredients/1"),
    ("sections", "/admin/sections"),
    ("categories", "/admin/categories"),
    ("review", "/admin/review"),
    ("mealie", "/admin/mealie"),
    ("settings", "/admin/settings"),
    ("recipes", "/recipes"),
    ("recipe-new", "/recipes/new"),
    ("recipe-import", "/recipes/import"),
    ("recipe", "/recipes/1"),
    ("cook", "/recipes/1/cook"),
    ("recipe-edit", "/recipes/1/edit"),
    ("shopping", "/shopping"),
    ("plan", "/plan"),
    ("not-found", "/browser-test-not-found"),
];

const WIDTHS: &[u32] = &[320, 390, 600, 768, 900, 1024, 1440];
const CHECKS: &str = r#"(() => {
  const errors = [];
  if (document.documentElement.scrollWidth > innerWidth) errors.push('horizontal overflow');
  if (document.querySelectorAll('h1').length !== 1) errors.push('expected exactly one h1');
  if (!document.querySelector('main')) errors.push('missing main landmark');
  for (const control of document.querySelectorAll('input,select,textarea')) {
    if (control.type === 'hidden') continue;
    if (!control.labels?.length && !control.getAttribute('aria-label') && !control.getAttribute('aria-labelledby')) errors.push('unlabelled form control');
  }
  for (const nav of document.querySelectorAll('nav')) if (!nav.getAttribute('aria-label') && !nav.getAttribute('aria-labelledby')) errors.push('unlabelled navigation landmark');
  for (const button of document.querySelectorAll('button,[role=button]')) if (!button.textContent.trim() && !button.getAttribute('aria-label') && !button.getAttribute('aria-labelledby')) errors.push('unlabelled button');
  for (const link of document.querySelectorAll('a[href]')) if (!link.textContent.trim() && !link.getAttribute('aria-label') && !link.getAttribute('aria-labelledby')) errors.push('unlabelled link');
  for (const invalid of document.querySelectorAll('[aria-invalid=true]')) {
    const described = (invalid.getAttribute('aria-describedby') || '').split(/\s+/).filter(Boolean);
    if (!described.some(id => document.getElementById(id))) errors.push('invalid control lacks an error association');
  }
  for (const status of document.querySelectorAll('[role=status],[role=alert]')) if (!status.textContent.trim()) errors.push('empty screen-reader status');
  for (const control of document.querySelectorAll('button,a[href],input,select,textarea,[role=button]')) {
    let target = control;
    if (control.matches('input[type=checkbox],input[type=radio]')) {
      const label = control.closest('label');
      if (!label) errors.push('checkbox or radio is not wrapped by a label');
      else target = label;
    }
    const box = target.getBoundingClientRect();
    if (!box.width || !box.height || getComputedStyle(control).visibility === 'hidden') continue;
    const minimum = control.matches('.cook-return,.cook-nav a,.cook-toggle,.timer-start,.timer-summary,.timer-row button,.cook-dot') ? 56 : 44;
    const prose = control.closest('p,li');
    const inlineTextLink = control.matches('a[href]') && prose && prose.textContent.trim() !== control.textContent.trim();
    if (!inlineTextLink && (box.width < 43.5 || box.height < minimum - 0.5)) errors.push(`small target ${control.outerHTML.slice(0,80)} ${Math.round(box.width)}x${Math.round(box.height)}`);
    let node = control.parentElement;
    let horizontalScroller = false;
    while (node) {
      if (['auto','scroll'].includes(getComputedStyle(node).overflowX)) horizontalScroller = true;
      node = node.parentElement;
    }
    if (!horizontalScroller && (box.left < 0 || box.right > innerWidth)) errors.push('interactive control clipped horizontally');
  }
  const active = document.querySelector('[aria-current="page"]');
  if (document.querySelector('nav') && !active && location.pathname !== '/browser-test-not-found') errors.push('missing active navigation item');
  const luminance = color => {
    const channels = color.match(/[\d.]+/g)?.slice(0,3).map(Number);
    if (!channels || channels.length !== 3) return null;
    const linear = channels.map(value => { value /= 255; return value <= .04045 ? value / 12.92 : ((value + .055) / 1.055) ** 2.4; });
    return .2126 * linear[0] + .7152 * linear[1] + .0722 * linear[2];
  };
  for (const text of document.querySelectorAll('main p,main li,main label,main h1,main h2,main h3,main button,nav a')) {
    const style = getComputedStyle(text);
    if (!text.textContent.trim()) continue;
    let node = text;
    let background = 'rgba(0, 0, 0, 0)';
    while (node && (background.endsWith(', 0)') || background === 'transparent')) {
      background = getComputedStyle(node).backgroundColor;
      node = node.parentElement;
    }
    const foregroundLum = luminance(style.color), backgroundLum = luminance(background);
    if (foregroundLum !== null && backgroundLum !== null) {
      const ratio = (Math.max(foregroundLum, backgroundLum) + .05) / (Math.min(foregroundLum, backgroundLum) + .05);
      const large = parseFloat(style.fontSize) >= 24 || (parseFloat(style.fontSize) >= 18.66 && Number(style.fontWeight) >= 700);
      if (ratio < (large ? 3 : 4.5)) errors.push(`low contrast ${ratio.toFixed(2)}:1`);
    }
  }
  return errors;
})()"#;

/// Runs route, responsive layout, interaction and screenshot checks against a served hydrated app.
///
/// # Errors
///
/// Returns an error when Chrome cannot start, the page fails a check, or an artifact cannot be saved.
pub async fn run(
    chrome: &Path,
    base_url: &str,
    language: &str,
    output: &Path,
) -> Result<(), String> {
    fs::create_dir_all(output).map_err(|error| error.to_string())?;
    let (mut browser, page, handler_task) = start_browser(chrome, base_url).await?;
    sign_in(&page).await?;
    let (mut failures, report) = check_routes(&page, base_url, language, output).await?;
    exercise_interactions(&page, base_url, output, &mut failures).await?;
    fs::write(output.join("report.csv"), report).map_err(|error| error.to_string())?;
    browser.close().await.map_err(|error| error.to_string())?;
    handler_task.abort();
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}

async fn start_browser(
    chrome: &Path,
    base_url: &str,
) -> Result<(Browser, Page, tokio::task::JoinHandle<()>), String> {
    let config = BrowserConfig::builder()
        .chrome_executable(chrome)
        .no_sandbox()
        .build()?;
    let (browser, mut handler) = Browser::launch(config)
        .await
        .map_err(|error| error.to_string())?;
    let handler_task = tokio::spawn(async move {
        while let Some(event) = handler.next().await {
            if event.is_err() {
                break;
            }
        }
    });
    let page = browser
        .new_page("about:blank")
        .await
        .map_err(|error| error.to_string())?;
    page.execute(chromiumoxide::cdp::browser_protocol::network::SetCacheDisabledParams::new(true))
        .await
        .map_err(|error| error.to_string())?;
    page.evaluate_on_new_document("window.__crabbageCls=0; new PerformanceObserver(list=>{for(const entry of list.getEntries()){if(!entry.hadRecentInput)window.__crabbageCls+=entry.value}}).observe({type:'layout-shift',buffered:true})")
        .await.map_err(|error| error.to_string())?;
    page.goto(base_url)
        .await
        .map_err(|error| error.to_string())?;
    Ok((browser, page, handler_task))
}

async fn sign_in(page: &Page) -> Result<(), String> {
    tokio::time::sleep(Duration::from_millis(700)).await;
    page.find_element("a[href*='user=admin']")
        .await
        .map_err(|error| format!("fake issuer sign-in link missing: {error}"))?
        .click()
        .await
        .map_err(|error| error.to_string())?;
    wait_for_hydration(page).await
}

pub(super) async fn wait_for_hydration(page: &Page) -> Result<(), String> {
    for _ in 0..200 {
        if let Ok(value) = page
            .evaluate("document.documentElement.dataset.crabbageHydrated === 'true'")
            .await
            && let Ok(hydrated) = value.into_value::<bool>()
            && hydrated
        {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    Err("application did not hydrate within 20 seconds".to_owned())
}

async fn check_routes(
    page: &Page,
    base_url: &str,
    language: &str,
    output: &Path,
) -> Result<(Vec<String>, String), String> {
    let mut failures = Vec::new();
    let mut report = String::from("route,width,height,theme,language,cls,asset_bytes\n");
    for (name, route) in ROUTES {
        for width in WIDTHS {
            capture_route(
                page,
                CheckSettings {
                    base_url,
                    name,
                    route,
                    width: *width,
                    height: 900,
                    theme: "light",
                    language,
                    output,
                },
                &mut failures,
                &mut report,
            )
            .await?;
        }
        capture_route(
            page,
            CheckSettings {
                base_url,
                name,
                route,
                width: 844,
                height: 390,
                theme: "light",
                language,
                output,
            },
            &mut failures,
            &mut report,
        )
        .await?;
        for (theme, width) in [("dark", 390), ("dark", 1440)] {
            capture_route(
                page,
                CheckSettings {
                    base_url,
                    name,
                    route,
                    width,
                    height: 900,
                    theme,
                    language,
                    output,
                },
                &mut failures,
                &mut report,
            )
            .await?;
        }
    }
    Ok((failures, report))
}

async fn capture_route(
    page: &Page,
    settings: CheckSettings<'_>,
    failures: &mut Vec<String>,
    report: &mut String,
) -> Result<(), String> {
    check_page(page, &settings, failures, report).await
}

async fn check_page(
    page: &Page,
    settings: &CheckSettings<'_>,
    failures: &mut Vec<String>,
    report: &mut String,
) -> Result<(), String> {
    navigate_and_set_view(page, settings).await?;
    check_accessibility(page, settings, failures).await?;
    record_metrics(page, settings, report, failures).await?;
    // NOTE: full-page screenshots leave the viewport resized, so check reflow first.
    check_text_reflow(page, settings, failures).await?;
    save_screenshot(page, settings).await
}

async fn navigate_and_set_view(page: &Page, settings: &CheckSettings<'_>) -> Result<(), String> {
    let CheckSettings {
        base_url,
        route,
        width,
        height,
        theme,
        language,
        ..
    } = settings;
    let url = format!("{base_url}{route}?lang={language}");
    page.goto(url.as_str())
        .await
        .map_err(|error| error.to_string())?;
    let viewport = SetDeviceMetricsOverrideParams::builder()
        .width(*width)
        .height(*height)
        .device_scale_factor(1.0)
        .mobile(*width <= 600)
        .build()?;
    page.execute(viewport)
        .await
        .map_err(|error| error.to_string())?;
    let media = SetEmulatedMediaParams::builder()
        .features(vec![MediaFeature::new("prefers-color-scheme", *theme)])
        .build();
    page.execute(media)
        .await
        .map_err(|error| error.to_string())?;
    wait_for_hydration(page).await?;
    page.evaluate(format!("document.documentElement.dataset.browserTheme='{theme}'; document.documentElement.lang='{language}'; document.documentElement.style.setProperty('scroll-behavior','auto'); if(!document.querySelector('#browser-stability')){{const s=document.createElement('style');s.id='browser-stability';s.textContent='*,*::before,*::after{{animation:none!important;transition:none!important;caret-color:transparent!important}}';document.head.append(s)}}"))
        .await.map_err(|error| error.to_string())?;
    tokio::time::sleep(Duration::from_millis(120)).await;
    Ok(())
}

async fn check_accessibility(
    page: &Page,
    settings: &CheckSettings<'_>,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    let CheckSettings {
        route,
        width,
        height,
        ..
    } = settings;
    let errors: Vec<String> = page
        .evaluate(CHECKS)
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    failures.extend(
        errors
            .into_iter()
            .map(|error| format!("{route} {width}x{height}: {error}")),
    );
    // NOTE: Element::press_key dispatches the key to the page; focusing body first is a no-op.
    page.find_element("body")
        .await
        .map_err(|error| error.to_string())?
        .press_key("Tab")
        .await
        .map_err(|error| error.to_string())?;
    let focus_visible: bool = page.evaluate("(() => { const e=document.activeElement,s=getComputedStyle(e); return !!e && (s.outlineStyle!=='none' && parseFloat(s.outlineWidth)>0 || s.boxShadow!=='none'); })()")
        .await.map_err(|error| error.to_string())?.into_value().map_err(|error| error.to_string())?;
    if !focus_visible {
        failures.push(format!("{route}: keyboard focus indicator not visible"));
    }
    page.evaluate("document.activeElement?.blur()")
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}

async fn record_metrics(
    page: &Page,
    settings: &CheckSettings<'_>,
    report: &mut String,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    let CheckSettings {
        route,
        width,
        height,
        theme,
        language,
        ..
    } = settings;
    let metrics: Value = page.evaluate("({cls: window.__crabbageCls||0, bytes: performance.getEntriesByType('resource').reduce((n,e)=>n+(e.transferSize||0),0)})").await.map_err(|error| error.to_string())?.into_value().map_err(|error| error.to_string())?;
    let cls = metrics
        .get("cls")
        .and_then(Value::as_f64)
        .unwrap_or_default();
    let bytes = metrics
        .get("bytes")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    if cls >= 0.1 {
        failures.push(format!("{route}: CLS {cls:.3} >= 0.1"));
    }
    writeln!(
        report,
        "{route},{width},{height},{theme},{language},{cls:.4},{bytes}"
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

async fn save_screenshot(page: &Page, settings: &CheckSettings<'_>) -> Result<(), String> {
    let CheckSettings {
        output,
        name,
        width,
        height,
        theme,
        language,
        ..
    } = settings;
    let directory = output.join(name);
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let file = directory.join(format!("{width}x{height}-{theme}-{language}.png"));
    page.save_screenshot(
        ScreenshotParams::builder()
            .format(CaptureScreenshotFormat::Png)
            .full_page(true)
            .build(),
        file,
    )
    .await
    .map_err(|error| error.to_string())?;
    Ok(())
}

async fn check_text_reflow(
    page: &Page,
    settings: &CheckSettings<'_>,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    let CheckSettings { route, width, .. } = settings;
    if *width == 320 {
        page.evaluate("document.documentElement.style.fontSize='200%'")
            .await
            .map_err(|error| error.to_string())?;
        let overflow: String = page
            .evaluate("document.documentElement.scrollWidth > innerWidth ? [...document.querySelectorAll('body *')].filter(e => e.getBoundingClientRect().right > innerWidth + 1).slice(0, 4).map(e => `${e.tagName.toLowerCase()}.${e.className} ${Math.round(e.getBoundingClientRect().right)}px`).join(', ') || 'unknown element' : ''")
            .await
            .map_err(|error| error.to_string())?
            .into_value()
            .map_err(|error| error.to_string())?;
        if !overflow.is_empty() {
            failures.push(format!("{route}: overflow at 200% text size: {overflow}"));
        }
        page.evaluate("document.documentElement.style.fontSize=''")
            .await
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

struct CheckSettings<'a> {
    base_url: &'a str,
    name: &'a str,
    route: &'a str,
    width: u32,
    height: u32,
    theme: &'a str,
    language: &'a str,
    output: &'a Path,
}
