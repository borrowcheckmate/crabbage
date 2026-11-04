use leptos::ev::SubmitEvent;

/// Sends a hydrated form's POST without leaving the page, so its scroll position and state stay.
/// Returns whether the request was taken over. The form is reset once the server accepts it; when
/// the request fails or is refused the form is submitted natively, so the server's answer shows.
#[cfg(feature = "hydrate")]
pub(crate) fn submit_in_background(event: &SubmitEvent) -> bool {
    use wasm_bindgen::JsCast;
    let Some(form) = event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlFormElement>().ok())
    else {
        return false;
    };
    let Some(body) = web_sys::FormData::new_with_form(&form)
        .and_then(|data| web_sys::UrlSearchParams::new_with_str_sequence_sequence(&data))
        .ok()
    else {
        return false;
    };
    let init = web_sys::RequestInit::new();
    init.set_method("POST");
    init.set_body(&body);
    init.set_redirect(web_sys::RequestRedirect::Manual);
    let response = leptos::prelude::window().fetch_with_str_and_init(&form.action(), &init);
    settle(&response, form);
    event.prevent_default();
    true
}

#[cfg(feature = "hydrate")]
fn settle(response: &js_sys::Promise, form: web_sys::HtmlFormElement) {
    use wasm_bindgen::{JsCast, JsValue, closure::Closure};
    let outcome = Closure::once_into_js(move |outcome: JsValue| {
        let delivered = outcome
            .dyn_ref::<web_sys::Response>()
            .is_some_and(|response| {
                response.ok() || response.type_() == web_sys::ResponseType::Opaqueredirect
            });
        if delivered {
            form.reset();
        } else {
            let _submitted = form.submit();
        }
    });
    if let Some(then) = js_sys::Reflect::get(response, &JsValue::from_str("then"))
        .ok()
        .and_then(|then| then.dyn_into::<js_sys::Function>().ok())
    {
        let _chained = then.call2(response, &outcome, &outcome);
    }
}

/// Without hydration a form always submits natively.
#[cfg(not(feature = "hydrate"))]
pub(crate) const fn submit_in_background(_event: &SubmitEvent) -> bool {
    false
}
