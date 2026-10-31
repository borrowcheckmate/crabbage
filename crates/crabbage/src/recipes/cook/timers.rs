//! The cook page's timers. A timer ends at a moment in milliseconds since the Unix epoch, so it
//! stays right while the screen sleeps. Pure: the browser side is in `tray_client.rs`.

use crabbage_domain::durations::detect_durations;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Timer {
    pub(crate) id: u64,
    pub(crate) label: String,
    pub(crate) ends_at_ms: f64,
}

pub(crate) fn start(timers: &mut Vec<Timer>, label: String, seconds: u32, now_ms: f64) {
    let id = timers
        .iter()
        .map(|timer| timer.id)
        .max()
        .map_or(1, |last| last.saturating_add(1));
    timers.push(Timer {
        id,
        label,
        ends_at_ms: f64::mul_add(f64::from(seconds), 1000.0, now_ms),
    });
}

pub(crate) fn cancel(timers: &mut Vec<Timer>, id: u64) {
    timers.retain(|timer| timer.id != id);
}

pub(crate) fn remaining_ms(timer: &Timer, now_ms: f64) -> f64 {
    (timer.ends_at_ms - now_ms).max(0.0)
}

pub(crate) fn finished(timer: &Timer, now_ms: f64) -> bool {
    timer.ends_at_ms <= now_ms
}

pub(crate) fn soonest_first(timers: &[Timer]) -> Vec<Timer> {
    let mut sorted = timers.to_vec();
    sorted.sort_by(|first, second| first.ends_at_ms.total_cmp(&second.ends_at_ms));
    sorted
}

/// The time left as `mm:ss`, or `h:mm:ss` from an hour, rounded up to the second.
pub(crate) fn clock(ms: f64) -> String {
    let total = format!("{:.0}", (ms / 1000.0).ceil())
        .parse::<u64>()
        .unwrap_or(0);
    let (hours, rest) = (total / 3600, total % 3600);
    let (minutes, seconds) = (rest / 60, rest % 60);
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

/// The timers offered for a step: its stored duration first, then those named in its text, each
/// once. Zero and out-of-range amounts are left out.
pub(crate) fn step_durations(text: &str, stored: Option<i64>) -> Vec<u32> {
    let stored = stored.and_then(|seconds| u32::try_from(seconds).ok());
    let named = detect_durations(text)
        .into_iter()
        .filter_map(|duration| u32::try_from(duration.as_secs()).ok());
    let mut unique: Vec<u32> = Vec::new();
    for seconds in stored
        .into_iter()
        .chain(named)
        .filter(|seconds| *seconds > 0)
    {
        if !unique.contains(&seconds) {
            unique.push(seconds);
        }
    }
    unique
}

#[cfg(any(feature = "hydrate", test))]
pub(crate) fn encode(timers: &[Timer]) -> String {
    serde_json::to_string(timers).unwrap_or_default()
}

#[cfg(any(feature = "hydrate", test))]
pub(crate) fn decode(text: &str) -> Vec<Timer> {
    serde_json::from_str(text).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::step_durations;
    use super::{
        Timer, cancel, clock, decode, encode, finished, remaining_ms, soonest_first, start,
    };

    fn timer(id: u64, ends_at_ms: f64) -> Timer {
        Timer {
            id,
            label: format!("Step {id}"),
            ends_at_ms,
        }
    }

    #[test]
    fn the_same_label_started_twice_gives_two_timers() {
        let mut timers = Vec::new();
        start(&mut timers, "Step 2".to_owned(), 60, 0.0);
        start(&mut timers, "Step 2".to_owned(), 60, 0.0);
        assert_eq!(timers.len(), 2);
        assert_ne!(timers[0].id, timers[1].id);
    }

    #[test]
    fn several_timers_run_at_once_and_end_at_their_own_times() {
        let mut timers = Vec::new();
        start(&mut timers, "Step 1".to_owned(), 120, 1_000.0);
        start(&mut timers, "Step 3".to_owned(), 30, 1_000.0);
        start(&mut timers, "Step 2".to_owned(), 3600, 5_000.0);
        assert_eq!(timers.len(), 3);
        assert_eq!(timers[0].ends_at_ms, 121_000.0);
        assert_eq!(timers[1].ends_at_ms, 31_000.0);
        assert_eq!(timers[2].ends_at_ms, 3_605_000.0);
    }

    #[test]
    fn timers_are_listed_soonest_first() {
        let timers = vec![timer(1, 300.0), timer(2, 100.0), timer(3, 200.0)];
        let ids: Vec<u64> = soonest_first(&timers).iter().map(|t| t.id).collect();
        assert_eq!(ids, [2, 3, 1]);
        assert_eq!(timers[0].id, 1);
    }

    #[test]
    fn remaining_time_counts_down_and_stops_at_zero() {
        let running = timer(1, 10_000.0);
        assert_eq!(remaining_ms(&running, 4_000.0), 6_000.0);
        assert!(remaining_ms(&running, 10_000.0) <= 0.0);
        assert!(remaining_ms(&running, 99_000.0) <= 0.0);
    }

    #[test]
    fn a_timer_is_finished_once_its_end_is_reached() {
        let running = timer(1, 10_000.0);
        assert!(!finished(&running, 9_999.0));
        assert!(finished(&running, 10_000.0));
        assert!(finished(&running, 20_000.0));
    }

    #[test]
    fn cancelling_removes_only_that_timer() {
        let mut timers = vec![timer(1, 1.0), timer(2, 2.0), timer(3, 3.0)];
        cancel(&mut timers, 2);
        let ids: Vec<u64> = timers.iter().map(|t| t.id).collect();
        assert_eq!(ids, [1, 3]);
        cancel(&mut timers, 9);
        assert_eq!(timers.len(), 2);
    }

    #[test]
    fn ids_are_not_reused_while_a_timer_runs() {
        let mut timers = vec![timer(4, 1.0)];
        start(&mut timers, "Step 2".to_owned(), 60, 0.0);
        assert_eq!(timers[1].id, 5);
    }

    #[test]
    fn the_clock_shows_minutes_and_seconds_then_hours() {
        assert_eq!(clock(1_500_000.0), "25:00");
        assert_eq!(clock(3_599_000.0), "59:59");
        assert_eq!(clock(3_600_000.0), "1:00:00");
        assert_eq!(clock(3_601_000.0), "1:00:01");
        assert_eq!(clock(0.0), "00:00");
        assert_eq!(clock(-5.0), "00:00");
    }

    #[test]
    fn the_clock_rounds_a_part_second_up() {
        assert_eq!(clock(400.0), "00:01");
        assert_eq!(clock(1_000.0), "00:01");
        assert_eq!(clock(1_001.0), "00:02");
    }

    #[test]
    fn a_step_offers_its_stored_duration_and_the_ones_in_its_text_once() {
        assert_eq!(step_durations("Bake for 25 minutes", None), [1500]);
        assert_eq!(step_durations("Bake for 25 minutes", Some(1500)), [1500]);
        assert_eq!(step_durations("Fry until golden", Some(240)), [240]);
        assert_eq!(step_durations("Bake 1 hour", Some(120)), [120, 3600]);
        assert_eq!(step_durations("Laat 1 uur rusten", None), [3600]);
    }

    #[test]
    fn a_step_without_a_duration_offers_none() {
        assert_eq!(step_durations("Stir in the flour", None), Vec::<u32>::new());
        assert_eq!(
            step_durations("Stir in the flour", Some(0)),
            Vec::<u32>::new()
        );
        assert_eq!(
            step_durations("Stir in the flour", Some(-5)),
            Vec::<u32>::new()
        );
    }

    #[test]
    fn timers_survive_the_session_storage_round_trip() {
        let timers = vec![timer(1, 1_500.5), timer(2, 9.0)];
        assert_eq!(decode(&encode(&timers)), timers);
    }

    #[test]
    fn stored_text_that_is_not_a_timer_list_is_ignored() {
        assert_eq!(decode("not json"), Vec::<Timer>::new());
        assert_eq!(decode(""), Vec::<Timer>::new());
    }
}
