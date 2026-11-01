use leptos::prelude::*;
use leptos_router::hooks::use_location;
#[cfg(feature = "hydrate")]
use wasm_bindgen::{JsCast, JsValue, closure::Closure};

#[cfg(feature = "hydrate")]
const SCROLL_KEY: &str = "__crabbage_scroll_v1";
#[cfg(feature = "hydrate")]
const ORIGINAL_STATE_KEY: &str = "__crabbage_original_state_v1";

#[cfg(feature = "hydrate")]
fn history() -> Option<web_sys::History> {
    window().history().ok()
}

#[cfg(feature = "hydrate")]
fn metadata(state: &JsValue) -> Option<JsValue> {
    let value = js_sys::Reflect::get(state, &JsValue::from_str(SCROLL_KEY)).ok()?;
    value.is_object().then_some(value)
}

#[cfg(feature = "hydrate")]
fn replace_scroll_state(x: f64, y: f64, initialize: bool) {
    let Some(history) = history() else { return };
    let Ok(state) = history.state() else { return };
    let old_meta = metadata(&state);
    if !initialize && old_meta.is_none() {
        return;
    }
    let object = js_sys::Object::new();
    if state.is_object() && !state.is_null() {
        let state_object: &js_sys::Object = state.unchecked_ref();
        drop(js_sys::Object::assign(&object, state_object));
    } else if !state.is_undefined() {
        drop(js_sys::Reflect::set(
            &object,
            &JsValue::from_str(ORIGINAL_STATE_KEY),
            &state,
        ));
    }
    let meta = old_meta.unwrap_or_else(|| {
        let meta = js_sys::Object::new();
        let id = format!(
            "{}-{}",
            js_sys::Date::now(),
            SCROLL_ENTRY_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        );
        drop(js_sys::Reflect::set(
            &meta,
            &JsValue::from_str("id"),
            &JsValue::from_str(&id),
        ));
        meta.into()
    });
    store_position(&meta, x, y);
    drop(js_sys::Reflect::set(
        &object,
        &JsValue::from_str(SCROLL_KEY),
        &meta,
    ));
    drop(history.replace_state_with_url(&object, "", None));
}

#[cfg(feature = "hydrate")]
fn store_position(meta: &JsValue, x: f64, y: f64) {
    for (key, value) in [("x", x), ("y", y)] {
        drop(js_sys::Reflect::set(
            meta,
            &JsValue::from_str(key),
            &JsValue::from_f64(value),
        ));
    }
    let height = document()
        .document_element()
        .map_or(0.0, |element| f64::from(element.scroll_height()));
    let width = window()
        .inner_width()
        .ok()
        .and_then(|value| value.as_f64())
        .unwrap_or(0.0);
    for (key, value) in [("height", height), ("width", width)] {
        drop(js_sys::Reflect::set(
            meta,
            &JsValue::from_str(key),
            &JsValue::from_f64(value),
        ));
    }
}

#[cfg(feature = "hydrate")]
fn finish_navigation(
    hash: &str,
    is_history_navigation: bool,
    set_is_history_navigation: WriteSignal<bool>,
    attempts: u16,
) -> bool {
    if is_history_navigation {
        if !hash.is_empty() {
            if let Some(target) = document().get_element_by_id(hash.trim_start_matches('#')) {
                target.scroll_into_view();
            }
        } else if let Some(meta) = history()
            .and_then(|history| history.state().ok())
            .and_then(|state| metadata(&state))
            && !restore_entry_position(&meta, attempts, set_is_history_navigation)
        {
            return false;
        }
    } else {
        replace_scroll_state(
            window().scroll_x().unwrap_or(0.0),
            window().scroll_y().unwrap_or(0.0),
            true,
        );
    }
    set_is_history_navigation.set(false);
    true
}

#[cfg(feature = "hydrate")]
fn restore_entry_position(
    meta: &JsValue,
    attempts: u16,
    set_is_history_navigation: WriteSignal<bool>,
) -> bool {
    let Ok(x) = js_sys::Reflect::get(meta, &JsValue::from_str("x")) else {
        return true;
    };
    let Ok(y) = js_sys::Reflect::get(meta, &JsValue::from_str("y")) else {
        return true;
    };
    let x = x.as_f64().unwrap_or(0.0);
    let y = y.as_f64().unwrap_or(0.0);
    let current_height = document()
        .document_element()
        .map_or(0.0, |element| f64::from(element.scroll_height()));
    let saved_height = js_sys::Reflect::get(meta, &JsValue::from_str("height"))
        .ok()
        .and_then(|value| value.as_f64())
        .unwrap_or(0.0);
    let current_width = window()
        .inner_width()
        .ok()
        .and_then(|value| value.as_f64())
        .unwrap_or(0.0);
    let saved_width = js_sys::Reflect::get(meta, &JsValue::from_str("width"))
        .ok()
        .and_then(|value| value.as_f64())
        .unwrap_or(current_width);
    let waiting_for_content =
        (saved_width - current_width).abs() < 1.0 && current_height + 1.0 < saved_height;
    let max_scroll = window()
        .inner_height()
        .ok()
        .and_then(|height| height.as_f64())
        .map_or(0.0, |height| (current_height - height).max(0.0));
    if y > max_scroll + 1.0 || waiting_for_content {
        if attempts < 360 {
            return false;
        }
        set_is_history_navigation.set(false);
        return true;
    }
    window().scroll_to_with_x_and_y(x, y);
    true
}

#[component]
pub(crate) fn RouteFocus(
    is_routing: ReadSignal<bool>,
    is_history_navigation: ReadSignal<bool>,
) -> impl IntoView {
    let location = use_location();
    let previous_path = StoredValue::new(location.pathname.get_untracked());
    let focus_pending = StoredValue::new(false);
    Effect::new(move |_| {
        let path = location.pathname.get();
        let routing = is_routing.get();
        if previous_path.get_value() != path {
            previous_path.set_value(path);
            if is_history_navigation.get_untracked() || !location.hash.get_untracked().is_empty() {
                focus_pending.set_value(false);
                return;
            }
            focus_pending.set_value(true);
            #[cfg(feature = "hydrate")]
            request_animation_frame(move || {
                if !is_routing.get_untracked() {
                    focus_pending.set_value(false);
                    focus_main();
                }
            });
        } else if focus_pending.get_value() && !routing {
            focus_pending.set_value(false);
            #[cfg(feature = "hydrate")]
            focus_main();
        }
    });
    view! { <span hidden=true></span> }
}

#[component]
pub(crate) fn HistoryScroll(
    is_routing: ReadSignal<bool>,
    is_history_navigation: ReadSignal<bool>,
    set_is_history_navigation: WriteSignal<bool>,
) -> impl IntoView {
    #[cfg(feature = "hydrate")]
    let (history_generation, set_history_generation) = signal(0_u64);
    #[cfg(feature = "hydrate")]
    let (user_intent_generation, set_user_intent_generation) = signal(0_u64);
    #[cfg(feature = "hydrate")]
    install_history_tracking(
        is_history_navigation,
        set_is_history_navigation,
        history_generation,
        set_history_generation,
    );
    #[cfg(feature = "hydrate")]
    install_user_intent_tracking(user_intent_generation, set_user_intent_generation);
    #[cfg(feature = "hydrate")]
    track_location(
        is_routing,
        is_history_navigation,
        set_is_history_navigation,
        history_generation,
        user_intent_generation,
    );
    #[cfg(not(feature = "hydrate"))]
    let _: (ReadSignal<bool>, ReadSignal<bool>, WriteSignal<bool>) =
        (is_routing, is_history_navigation, set_is_history_navigation);
    view! { <span hidden=true></span> }
}

#[cfg(feature = "hydrate")]
fn install_history_tracking(
    is_history_navigation: ReadSignal<bool>,
    set_is_history_navigation: WriteSignal<bool>,
    history_generation: ReadSignal<u64>,
    set_history_generation: WriteSignal<u64>,
) {
    use leptos::__reexports::send_wrapper::SendWrapper;
    use web_sys::Event;

    let Some(history) = history() else { return };
    let previous_restoration =
        js_sys::Reflect::get(history.as_ref(), &JsValue::from_str("scrollRestoration"))
            .unwrap_or(JsValue::UNDEFINED);
    drop(js_sys::Reflect::set(
        history.as_ref(),
        &JsValue::from_str("scrollRestoration"),
        &JsValue::from_str("manual"),
    ));
    replace_scroll_state(
        window().scroll_x().unwrap_or(0.0),
        window().scroll_y().unwrap_or(0.0),
        true,
    );
    let popstate = SendWrapper::new(Closure::<dyn FnMut(Event)>::new(move |_event| {
        set_history_generation.set(history_generation.get_untracked().wrapping_add(1));
        set_is_history_navigation.set(true);
    }));
    drop(window().add_event_listener_with_callback_and_bool(
        "popstate",
        popstate.as_ref().unchecked_ref(),
        true,
    ));
    let scroll = SendWrapper::new(Closure::<dyn FnMut(Event)>::new(move |_event| {
        if !is_history_navigation.get_untracked() {
            replace_scroll_state(
                window().scroll_x().unwrap_or(0.0),
                window().scroll_y().unwrap_or(0.0),
                false,
            );
        }
    }));
    drop(window().add_event_listener_with_callback("scroll", scroll.as_ref().unchecked_ref()));
    install_pre_navigation_saver();
    let cleanup_history = SendWrapper::new(history);
    let cleanup_restoration = SendWrapper::new(previous_restoration);
    on_cleanup(move || {
        drop(window().remove_event_listener_with_callback_and_bool(
            "popstate",
            popstate.as_ref().unchecked_ref(),
            true,
        ));
        drop(
            window().remove_event_listener_with_callback("scroll", scroll.as_ref().unchecked_ref()),
        );
        drop(js_sys::Reflect::set(
            cleanup_history.as_ref(),
            &JsValue::from_str("scrollRestoration"),
            &cleanup_restoration,
        ));
    });
}

#[cfg(feature = "hydrate")]
fn install_pre_navigation_saver() {
    use leptos::__reexports::send_wrapper::SendWrapper;
    use web_sys::Event;

    let save = SendWrapper::new(Closure::<dyn FnMut(Event)>::new(move |_event| {
        replace_scroll_state(
            window().scroll_x().unwrap_or(0.0),
            window().scroll_y().unwrap_or(0.0),
            false,
        );
    }));
    for event in ["click", "submit"] {
        drop(window().add_event_listener_with_callback_and_bool(
            event,
            save.as_ref().unchecked_ref(),
            true,
        ));
    }
    on_cleanup(move || {
        for event in ["click", "submit"] {
            drop(window().remove_event_listener_with_callback_and_bool(
                event,
                save.as_ref().unchecked_ref(),
                true,
            ));
        }
    });
}

#[cfg(feature = "hydrate")]
fn install_user_intent_tracking(
    user_intent_generation: ReadSignal<u64>,
    set_user_intent_generation: WriteSignal<u64>,
) {
    use leptos::__reexports::send_wrapper::SendWrapper;
    use web_sys::Event;

    let user_intent = SendWrapper::new(Closure::<dyn FnMut(Event)>::new(move |_event| {
        set_user_intent_generation.set(user_intent_generation.get_untracked().wrapping_add(1));
    }));
    for event in ["wheel", "touchstart", "keydown", "pointerdown"] {
        drop(
            window().add_event_listener_with_callback(event, user_intent.as_ref().unchecked_ref()),
        );
    }
    on_cleanup(move || {
        for event in ["wheel", "touchstart", "keydown", "pointerdown"] {
            drop(
                window().remove_event_listener_with_callback(
                    event,
                    user_intent.as_ref().unchecked_ref(),
                ),
            );
        }
    });
}

#[cfg(feature = "hydrate")]
#[derive(Clone, Copy)]
struct NavigationRestore {
    is_routing: ReadSignal<bool>,
    is_history_navigation: ReadSignal<bool>,
    set_is_history_navigation: WriteSignal<bool>,
    navigation_pending: StoredValue<bool>,
    pending_url: StoredValue<String>,
    history_generation: ReadSignal<u64>,
    user_intent_generation: ReadSignal<u64>,
}

#[cfg(feature = "hydrate")]
struct RestoreAttempt {
    url: String,
    history_generation: u64,
    user_intent_generation: u64,
    attempts: u16,
}

#[cfg(feature = "hydrate")]
fn track_location(
    is_routing: ReadSignal<bool>,
    is_history_navigation: ReadSignal<bool>,
    set_is_history_navigation: WriteSignal<bool>,
    history_generation: ReadSignal<u64>,
    user_intent_generation: ReadSignal<u64>,
) {
    let location = use_location();
    let previous_url = StoredValue::new(format!(
        "{}{}{}",
        location.pathname.get_untracked(),
        location.search.get_untracked(),
        location.hash.get_untracked()
    ));
    let navigation_pending = StoredValue::new(false);
    let pending_url = StoredValue::new(String::new());
    let restore = NavigationRestore {
        is_routing,
        is_history_navigation,
        set_is_history_navigation,
        navigation_pending,
        pending_url,
        history_generation,
        user_intent_generation,
    };
    Effect::new(move |_| {
        let url = format!(
            "{}{}{}",
            location.pathname.get(),
            location.search.get(),
            location.hash.get()
        );
        if previous_url.get_value() != url {
            previous_url.set_value(url.clone());
            navigation_pending.set_value(true);
            pending_url.set_value(url.clone());
        }
        if !is_routing.get() && (navigation_pending.get_value() || is_history_navigation.get()) {
            let location = location.clone();
            let expected_url = url;
            let expected_history_generation = history_generation.get();
            let expected_user_intent_generation = user_intent_generation.get_untracked();
            request_animation_frame(move || {
                finish_if_settled(
                    location,
                    restore,
                    RestoreAttempt {
                        url: expected_url,
                        history_generation: expected_history_generation,
                        user_intent_generation: expected_user_intent_generation,
                        attempts: 0,
                    },
                );
            });
        }
    });
}

#[cfg(feature = "hydrate")]
fn finish_if_settled(
    location: leptos_router::location::Location,
    restore: NavigationRestore,
    attempt: RestoreAttempt,
) {
    let current_url = format!(
        "{}{}{}",
        location.pathname.get_untracked(),
        location.search.get_untracked(),
        location.hash.get_untracked()
    );
    let navigation_changed = restore.navigation_pending.get_value();
    if restore.is_routing.get_untracked()
        || current_url != attempt.url
        || restore.history_generation.get_untracked() != attempt.history_generation
        || (navigation_changed && restore.pending_url.get_value() != current_url)
    {
        return;
    }
    if restore.user_intent_generation.get_untracked() != attempt.user_intent_generation {
        restore.set_is_history_navigation.set(false);
        return;
    }
    if navigation_changed {
        restore.navigation_pending.set_value(false);
        restore.pending_url.set_value(String::new());
    }
    if !finish_navigation(
        &location.hash.get_untracked(),
        restore.is_history_navigation.get_untracked(),
        restore.set_is_history_navigation,
        attempt.attempts,
    ) {
        request_animation_frame(move || {
            finish_if_settled(
                location,
                restore,
                RestoreAttempt {
                    attempts: attempt.attempts + 1,
                    ..attempt
                },
            );
        });
    }
}

#[cfg(feature = "hydrate")]
static SCROLL_ENTRY_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

#[cfg(feature = "hydrate")]
pub(crate) fn focus_main() {
    let Some(main) = document()
        .get_element_by_id("main-content")
        .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
    else {
        return;
    };
    let options = web_sys::FocusOptions::new();
    options.set_prevent_scroll(true);
    drop(main.focus_with_options(&options));
}
