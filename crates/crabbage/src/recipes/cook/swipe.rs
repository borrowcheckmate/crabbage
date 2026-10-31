//! The swipe on the cook page: a sideways swipe moves to the previous or the next step. The rule is
//! pure and tested here; the card connects it to the touch events in the browser.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Direction {
    Previous,
    Next,
}

const MIN_DISTANCE: f64 = 50.0;

pub(crate) fn swipe_target(
    start_x: f64,
    end_x: f64,
    start_y: f64,
    end_y: f64,
) -> Option<Direction> {
    let across = end_x - start_x;
    let along = end_y - start_y;
    if across.abs() < MIN_DISTANCE || across.abs() <= along.abs() {
        return None;
    }
    if across > 0.0 {
        Some(Direction::Previous)
    } else {
        Some(Direction::Next)
    }
}

#[cfg(feature = "hydrate")]
pub(crate) fn swipe_card(
    previous: Option<String>,
    next: Option<String>,
    content: leptos::prelude::AnyView,
) -> leptos::prelude::AnyView {
    use leptos::prelude::*;
    use leptos_router::NavigateOptions;
    use leptos_router::hooks::use_navigate;
    use web_sys::TouchEvent;

    let navigate = use_navigate();
    let start = RwSignal::new(None::<(f64, f64)>);
    let (before, after) = (previous.clone(), next.clone());
    let touch_start = move |event: TouchEvent| {
        if let Some(touch) = event.changed_touches().item(0) {
            start.set(Some((
                f64::from(touch.client_x()),
                f64::from(touch.client_y()),
            )));
        }
    };
    let touch_end = move |event: TouchEvent| {
        let (Some((x, y)), Some(touch)) = (start.get_untracked(), event.changed_touches().item(0))
        else {
            return;
        };
        start.set(None);
        let target = match swipe_target(
            x,
            f64::from(touch.client_x()),
            y,
            f64::from(touch.client_y()),
        ) {
            Some(Direction::Previous) => before.clone(),
            Some(Direction::Next) => after.clone(),
            None => None,
        };
        if let Some(href) = target {
            navigate(&href, NavigateOptions::default());
        }
    };
    view! {
        <div class="cook-instruction" data-prev=previous data-next=next on:touchstart=touch_start on:touchend=touch_end>
            {content}
        </div>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::{Direction, swipe_target};

    #[test]
    fn a_long_sideways_swipe_to_the_right_goes_to_the_previous_step() {
        assert_eq!(
            swipe_target(10.0, 90.0, 100.0, 104.0),
            Some(Direction::Previous)
        );
    }

    #[test]
    fn a_long_sideways_swipe_to_the_left_goes_to_the_next_step() {
        assert_eq!(
            swipe_target(200.0, 120.0, 100.0, 90.0),
            Some(Direction::Next)
        );
    }

    #[test]
    fn a_swipe_of_exactly_the_minimum_distance_counts() {
        assert_eq!(swipe_target(0.0, 50.0, 0.0, 0.0), Some(Direction::Previous));
        assert_eq!(swipe_target(50.0, 0.0, 0.0, 0.0), Some(Direction::Next));
    }

    #[test]
    fn a_short_swipe_does_nothing() {
        assert_eq!(swipe_target(0.0, 49.0, 0.0, 0.0), None);
        assert_eq!(swipe_target(100.0, 60.0, 0.0, 0.0), None);
    }

    #[test]
    fn a_mostly_vertical_move_does_nothing() {
        assert_eq!(swipe_target(0.0, 60.0, 0.0, 80.0), None);
        assert_eq!(swipe_target(0.0, 60.0, 0.0, 60.0), None);
        assert_eq!(swipe_target(0.0, -60.0, 0.0, -200.0), None);
    }
}
