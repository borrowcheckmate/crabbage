//! The browser side of the wake lock: the lock is requested while the cook page is open and released
//! when it closes. Hydrate only; the rules are in `wake.rs`.

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::{Closure, wasm_bindgen};
use web_sys::VisibilityState;

use super::wake::{WakeApi, WakeLock};

const EVENT: &str = "visibilitychange";

#[wasm_bindgen(inline_js = "
let sentinel = null;
let token = 0;
export function wakeSupported() { return 'wakeLock' in navigator; }
export function wakeRequest() {
  const mine = ++token;
  navigator.wakeLock.request('screen').then((lock) => {
    if (mine !== token) { lock.release().catch(() => {}); return; }
    sentinel = lock;
    lock.addEventListener('release', () => { if (sentinel === lock) sentinel = null; });
  }).catch(() => {});
}
export function wakeRelease() {
  token++;
  const lock = sentinel;
  sentinel = null;
  if (lock) lock.release().catch(() => {});
}
")]
extern "C" {
    #[wasm_bindgen(js_name = wakeSupported)]
    fn wake_supported() -> bool;
    #[wasm_bindgen(js_name = wakeRequest)]
    fn wake_request();
    #[wasm_bindgen(js_name = wakeRelease)]
    fn wake_release();
}

#[derive(Debug)]
pub(super) struct Browser;

impl WakeApi for Browser {
    fn supported(&self) -> bool {
        wake_supported()
    }

    fn request(&mut self) {
        wake_request();
    }

    fn release(&mut self) {
        wake_release();
    }
}

/// Keeps the screen awake for the page the component is mounted on. Browsers that drop the lock when
/// the page is hidden get it asked for again when the page is shown.
pub(super) fn attach() {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    let lock = StoredValue::new_local(WakeLock::new(Browser));
    let listener = StoredValue::new_local(Closure::<dyn Fn()>::new(move || {
        let visible = web_sys::window()
            .and_then(|window| window.document())
            .is_some_and(|document| document.visibility_state() == VisibilityState::Visible);
        lock.update_value(|wake| wake.visibility_changed(visible));
    }));
    let registered = listener.with_value(|change| {
        document.add_event_listener_with_callback(EVENT, change.as_ref().unchecked_ref())
    });
    if registered.is_err() {
        return;
    }
    let visible = document.visibility_state() == VisibilityState::Visible;
    lock.update_value(|wake| {
        wake.enter();
        wake.visibility_changed(visible);
    });
    on_cleanup(move || {
        lock.update_value(WakeLock::leave);
        let Some(document) = web_sys::window().and_then(|window| window.document()) else {
            return;
        };
        listener.with_value(|change| {
            drop(
                document
                    .remove_event_listener_with_callback(EVENT, change.as_ref().unchecked_ref()),
            );
        });
    });
}
