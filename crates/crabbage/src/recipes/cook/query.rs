//! The query values of the cook page: the step in view and the steps marked done, parsed from the
//! address and written back to it.

pub(crate) fn current_step(requested: Option<&str>, total: usize) -> usize {
    let last = total.max(1);
    requested
        .and_then(|text| text.parse::<usize>().ok())
        .map_or(1, |step| step.clamp(1, last))
}

pub(crate) fn done_steps(requested: Option<&str>, total: usize) -> Vec<usize> {
    let mut done: Vec<usize> = requested
        .unwrap_or_default()
        .split(',')
        .filter_map(|part| part.trim().parse::<usize>().ok())
        .filter(|step| (1..=total).contains(step))
        .collect();
    done.sort_unstable();
    done.dedup();
    done
}

pub(crate) fn toggled(done: &[usize], step: usize) -> Vec<usize> {
    let mut next: Vec<usize> = done
        .iter()
        .copied()
        .filter(|&marked| marked != step)
        .collect();
    if !done.contains(&step) {
        next.push(step);
    }
    next.sort_unstable();
    next
}

pub(crate) fn done_text(done: &[usize]) -> String {
    done.iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::{current_step, done_steps, done_text, toggled};

    #[test]
    fn the_step_is_one_when_missing_invalid_or_zero() {
        assert_eq!(current_step(None, 3), 1);
        assert_eq!(current_step(Some("abc"), 3), 1);
        assert_eq!(current_step(Some("-2"), 3), 1);
        assert_eq!(current_step(Some("0"), 3), 1);
        assert_eq!(current_step(Some(""), 3), 1);
    }

    #[test]
    fn the_step_is_the_requested_one_up_to_the_last() {
        assert_eq!(current_step(Some("2"), 3), 2);
        assert_eq!(current_step(Some("3"), 3), 3);
        assert_eq!(current_step(Some("9"), 3), 3);
    }

    #[test]
    fn a_recipe_without_steps_has_step_one() {
        assert_eq!(current_step(Some("4"), 0), 1);
    }

    #[test]
    fn done_steps_are_deduplicated_sorted_and_in_range() {
        assert_eq!(done_steps(Some("3,x,2,3,,0,9,-1, 1 "), 3), [1, 2, 3]);
        assert_eq!(done_steps(Some("2,1"), 3), [1, 2]);
        assert_eq!(done_steps(None, 3), Vec::<usize>::new());
        assert_eq!(done_steps(Some("1"), 0), Vec::<usize>::new());
    }

    #[test]
    fn toggling_adds_and_removes_a_step_in_order() {
        assert_eq!(toggled(&[2], 1), [1, 2]);
        assert_eq!(toggled(&[1, 2], 1), [2]);
        assert_eq!(toggled(&[], 3), [3]);
    }

    #[test]
    fn done_steps_are_written_comma_separated() {
        assert_eq!(done_text(&[1, 3]), "1,3");
        assert_eq!(done_text(&[]), "");
    }
}
