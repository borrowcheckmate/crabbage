use chromiumoxide::page::Page;
use serde_json::Value;

pub(super) async fn verify_hashed_assets(page: &Page) -> Result<(), String> {
    let assets: Vec<Value> = page
        .evaluate("(async () => { const urls = [...document.querySelectorAll('script[src],link[rel=stylesheet][href]')].map(element => element.src || element.href).filter(url => new URL(url).pathname.startsWith('/pkg/')); urls.push(...performance.getEntriesByType('resource').map(entry => entry.name).filter(url => { const path = new URL(url).pathname; return /^\\/pkg\\/[^/]+\\.(?:js|wasm)$/.test(path); })); const unique = [...new Set(urls)]; return Promise.all(unique.map(async url => { const response = await fetch(url, {cache:'reload'}); return {url, path:new URL(url).pathname, status:response.status, type:response.headers.get('content-type') || '', cache:response.headers.get('cache-control') || ''}; })); })()")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    let mut found_js = false;
    let mut found_wasm = false;
    let mut found_css = false;
    let mut failures = Vec::new();
    for asset in assets {
        if let Some(failure) = inspect_asset(&asset, &mut found_js, &mut found_wasm, &mut found_css)
        {
            failures.push(failure);
        }
    }
    for (extension, found) in [("js", found_js), ("wasm", found_wasm), ("css", found_css)] {
        if !found {
            failures.push(format!("hashed /pkg/ .{extension} asset was not requested"));
        }
    }
    let hydrated: bool = page
        .evaluate("document.documentElement.dataset.crabbageHydrated === 'true'")
        .await
        .map_err(|error| error.to_string())?
        .into_value()
        .map_err(|error| error.to_string())?;
    if !hydrated {
        failures.push("production app bundle did not hydrate".to_owned());
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}

fn inspect_asset(
    asset: &Value,
    found_js: &mut bool,
    found_wasm: &mut bool,
    found_css: &mut bool,
) -> Option<String> {
    let url = asset.get("url").and_then(Value::as_str).unwrap_or_default();
    let path = asset.get("path").and_then(Value::as_str)?;
    let primary = path
        .strip_prefix("/pkg/")
        .is_some_and(|asset| !asset.contains('/'));
    let extension = path.rsplit_once('.').map(|(_, extension)| extension)?;
    let expected_type = match extension {
        "js" => {
            *found_js |= primary;
            "javascript"
        }
        "wasm" => {
            *found_wasm |= primary;
            "application/wasm"
        }
        "css" => {
            *found_css |= primary;
            "text/css"
        }
        _ => return None,
    };
    if primary && !hashed_filename(path) {
        return Some(format!("asset URL is not content-hashed: {url}"));
    }
    let status = asset.get("status").and_then(Value::as_u64);
    let content_type = asset
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if status != Some(200) || !content_type.contains(expected_type) {
        return Some(format!(
            "asset did not load as expected: {url} ({status:?}, {content_type})"
        ));
    }
    if !primary
        && !asset
            .get("cache")
            .and_then(Value::as_str)
            .is_some_and(|cache| {
                cache
                    .split(',')
                    .any(|directive| directive.trim() == "no-cache")
            })
    {
        return Some(format!(
            "snippet asset does not require revalidation: {url}"
        ));
    }
    None
}

fn hashed_filename(path: &str) -> bool {
    path.rsplit_once('/')
        .and_then(|(_, name)| name.rsplit_once('.').map(|(stem, _)| stem))
        .and_then(|stem| stem.rsplit_once('.').map(|(_, hash)| hash))
        .is_some_and(|hash| {
            hash.len() >= 8
                && hash
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"_-".contains(&byte))
        })
}
