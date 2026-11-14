//! The browser side of the timers: the clock, the list kept in sessionStorage, and the alert when a
//! timer ends. Hydrate only; the list itself is in `timers.rs`.

use std::time::Duration;

use leptos::prelude::*;
use wasm_bindgen::prelude::wasm_bindgen;

use super::alert;
use super::timers::{Alarms, Timer, decode, encode};
use crate::i18n::t_with;

const KEY: &str = "crabbage.timers";

#[wasm_bindgen(js_namespace = Date)]
extern "C" {
    fn now() -> f64;
}

pub(super) fn now_ms() -> f64 {
    now()
}

/// Loads the stored timers and starts the one-second clock. Runs once the page is hydrated, so the
/// first render matches the server's empty tray. Each tick sounds the due alarms and announces the
/// first alarm of a timer that has ended.
pub(super) fn attach(
    timers: RwSignal<Vec<Timer>>,
    clock: RwSignal<f64>,
    announce: RwSignal<String>,
) {
    let alarms = StoredValue::new(Alarms::default());
    Effect::new(move |_| {
        clock.set(now_ms());
        timers.set(load());
    });
    let tick = move || {
        let at = now_ms();
        clock.set(at);
        let rang = alarms
            .try_update_value(|log| timers.with_untracked(|list| log.ring(list, at)))
            .unwrap_or_default();
        if let Some((timer, _)) = rang.iter().find(|(_, first)| *first) {
            announce.set(t_with("timer.announce.finished", &[("step", &timer.label)]));
        }
        if !rang.is_empty() {
            alert::alert();
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
