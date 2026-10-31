//! The browser side of the timers: the clock, the list kept in sessionStorage, and the alert when a
//! timer ends. Hydrate only; the list itself is in `timers.rs`.

use std::time::Duration;

use leptos::prelude::*;
use wasm_bindgen::prelude::wasm_bindgen;

use super::alert;
use super::timers::{Timer, decode, encode, finished};

const KEY: &str = "crabbage.timers";

#[wasm_bindgen(js_namespace = Date)]
extern "C" {
    fn now() -> f64;
}

pub(super) fn now_ms() -> f64 {
    now()
}

/// Loads the stored timers and starts the one-second clock. Runs once the page is hydrated, so the
/// first render matches the server's empty tray.
pub(super) fn attach(timers: RwSignal<Vec<Timer>>, clock: RwSignal<f64>) {
    let alerted = StoredValue::new(Vec::<u64>::new());
    Effect::new(move |_| {
        clock.set(now_ms());
        timers.set(load());
    });
    let tick = move || {
        let at = now_ms();
        clock.set(at);
        let fresh: Vec<u64> = timers.with_untracked(|list| {
            list.iter()
                .filter(|timer| finished(timer, at))
                .filter(|timer| !alerted.with_value(|ids| ids.contains(&timer.id)))
                .map(|timer| timer.id)
                .collect()
        });
        if !fresh.is_empty() {
            alert::alert();
            alerted.update_value(|ids| ids.extend(fresh));
        }
    };
    if let Ok(handle) = set_interval_with_handle(tick, Duration::from_secs(1)) {
        on_cleanup(move || handle.clear());
    }
}

pub(super) fn save(timers: &[Timer]) {
    storage().map(|storage| storage.set_item(KEY, &encode(timers)).ok());
}

fn load() -> Vec<Timer> {
    storage()
        .and_then(|storage| storage.get_item(KEY).ok().flatten())
        .map(|text| decode(&text))
        .unwrap_or_default()
}

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.session_storage().ok().flatten()
}
