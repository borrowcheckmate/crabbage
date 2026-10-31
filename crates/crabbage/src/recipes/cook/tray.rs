//! The timer tray and the start buttons of the cook page. The markup is the same in both builds;
//! the clock, the storage and the alert are in `tray_client.rs` and `alert.rs`.

use leptos::prelude::*;

use super::timers::{Timer, cancel, clock, finished, remaining_ms, soonest_first, start};
use crate::i18n::{t, t_with};

#[cfg(feature = "hydrate")]
use super::tray_client::{now_ms, save};

#[cfg(not(feature = "hydrate"))]
const fn now_ms() -> f64 {
    0.0
}

#[cfg(not(feature = "hydrate"))]
const fn save(_timers: &[Timer]) {}

/// The tray of running timers, one row each. Rendered empty on the server and filled in the browser.
#[component]
pub(crate) fn TimerTray(timers: RwSignal<Vec<Timer>>, now: RwSignal<f64>) -> impl IntoView {
    #[cfg(feature = "hydrate")]
    super::tray_client::attach(timers, now);
    view! {
        <details class="timer-tray">
            <summary class="timer-summary">
                {move || t_with("timer.tray.count", &[("count", &timers.with(|list| list.len().to_string()))])}
            </summary>
            <section class="timer-panel" aria-label=t("timer.tray")>
                <ul class="timer-list">
                    {move || {
                        soonest_first(&timers.get())
                        .into_iter()
                        .map(|timer| row(timer, now, timers))
                        .collect_view()
                    }}
                </ul>
            </section>
        </details>
    }
}

/// The button that starts a timer for `seconds` of step `step`.
pub(crate) fn start_button(timers: RwSignal<Vec<Timer>>, step: usize, seconds: u32) -> AnyView {
    let number = step.to_string();
    let label = t_with("timer.label", &[("step", &number)]);
    let started = label.clone();
    let minutes = seconds.div_ceil(60).to_string();
    let time = t_with("timer.duration.minutes", &[("minutes", &minutes)]);
    let aria = t_with("timer.start.step", &[("step", &number)]);
    view! {
        <button
            type="button"
            class="secondary timer-start"
            data-seconds=seconds.to_string()
            data-label=label
            aria-label=aria
            on:click=move |_| start_timer(timers, started.clone(), seconds)
        >
            <svg aria-hidden="true" viewBox="0 0 24 24"><circle cx="12" cy="13" r="8"/><path d="M12 9v4l2.5 1.5M9 2h6m-3 0v3"/></svg>
            {t_with("timer.start", &[("time", &time)])}
        </button>
    }
    .into_any()
}

fn start_timer(timers: RwSignal<Vec<Timer>>, label: String, seconds: u32) {
    #[cfg(feature = "hydrate")]
    super::alert::unlock();
    edit(timers, |list| start(list, label, seconds, now_ms()));
}

fn edit(timers: RwSignal<Vec<Timer>>, change: impl FnOnce(&mut Vec<Timer>)) {
    timers.update(change);
    save(&timers.get_untracked());
}

fn row(timer: Timer, now: RwSignal<f64>, timers: RwSignal<Vec<Timer>>) -> AnyView {
    let id = timer.id;
    let label = timer.label.clone();
    let shown = timer.clone();
    let done = Memo::new(move |_| now.with(|at| finished(&timer, *at)));
    let time = move || {
        now.with(|at| {
            if finished(&shown, *at) {
                t("timer.done").to_owned()
            } else {
                clock(remaining_ms(&shown, *at))
            }
        })
    };
    view! {
        <li class="timer-row" class:finished=move || done.get()>
            <span class="timer-label">{label}</span>
            <span class="timer-time">{time}</span>
            <button
                type="button"
                class="secondary"
                on:click=move |_| edit(timers, |list| cancel(list, id))
            >
                {move || if done.get() { t("timer.dismiss") } else { t("timer.cancel") }}
            </button>
        </li>
    }
    .into_any()
}
