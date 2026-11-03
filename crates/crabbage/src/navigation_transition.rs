//! Pending feedback and retained content for navigation within the current route.

use leptos::children::ViewFnOnce;
use leptos::prelude::*;

#[cfg(feature = "hydrate")]
#[derive(Clone, Copy)]
struct QueryPending(WriteSignal<bool>);

pub(crate) fn navigation_pending(routing: ReadSignal<bool>) -> Signal<bool> {
    #[cfg(feature = "hydrate")]
    {
        let (pending, set_pending) = signal(false);
        provide_context(QueryPending(set_pending));
        Signal::derive(move || {
            routing.try_get().unwrap_or_default() || pending.try_get().unwrap_or_default()
        })
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _: ReadSignal<bool> = routing;
        Signal::stored(false)
    }
}

#[component]
pub(crate) fn NavigationTransition(
    #[prop(optional, into)] fallback: ViewFnOnce,
    children: Children,
) -> impl IntoView {
    #[cfg(feature = "hydrate")]
    {
        let QueryPending(set_pending) = expect_context::<QueryPending>();
        on_cleanup(move || {
            let _cleared = set_pending.try_set(false);
        });
        view! {
            <Transition fallback=fallback set_pending=set_pending>
                {children()}
            </Transition>
        }
    }
    #[cfg(not(feature = "hydrate"))]
    {
        view! { <Transition fallback=fallback>{children()}</Transition> }
    }
}

#[cfg(test)]
mod tests {
    use super::navigation_pending;
    use leptos::prelude::*;

    #[test]
    fn pending_feedback_survives_router_source_disposal() {
        let root = Owner::new();
        root.with(|| {
            let router_owner = Owner::new();
            let (routing, _set_routing) = router_owner.with(|| signal(true));
            let view_owner = Owner::new();
            let pending = view_owner.with(|| navigation_pending(routing));
            assert!(pending.try_get().is_some());
            router_owner.cleanup();
            assert_eq!(pending.try_get(), Some(false));
        });
    }
}
