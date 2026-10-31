//! The addresses of the cook page. Each one keeps the servings and the steps marked done, so both
//! carry from step to step.

use super::query::{done_text, toggled};

pub(crate) struct Place {
    pub(crate) id: i64,
    pub(crate) servings: Option<i64>,
    pub(crate) done: Vec<usize>,
}

impl Place {
    pub(crate) fn step(&self, step: usize) -> String {
        self.address(step, &self.done)
    }

    pub(crate) fn toggle(&self, step: usize) -> String {
        self.address(step, &toggled(&self.done, step))
    }

    pub(crate) fn is_done(&self, step: usize) -> bool {
        self.done.contains(&step)
    }

    pub(crate) fn finish(&self) -> String {
        with_servings(format!("/recipes/{}", self.id), self.servings)
    }

    fn address(&self, step: usize, done: &[usize]) -> String {
        let mut parts = vec![format!("step={step}")];
        if let Some(count) = self.servings {
            parts.push(format!("servings={count}"));
        }
        if !done.is_empty() {
            parts.push(format!("done={}", done_text(done)));
        }
        format!("/recipes/{}/cook?{}", self.id, parts.join("&"))
    }
}

pub(crate) fn start_href(id: i64, servings: Option<i64>) -> String {
    with_servings(format!("/recipes/{id}/cook"), servings)
}

fn with_servings(path: String, servings: Option<i64>) -> String {
    match servings {
        Some(count) => format!("{path}?servings={count}"),
        None => path,
    }
}

#[cfg(test)]
mod tests {
    use super::{Place, start_href};

    #[test]
    fn the_addresses_keep_the_servings_and_the_done_steps() {
        let place = Place {
            id: 7,
            servings: Some(8),
            done: vec![1, 3],
        };
        assert_eq!(place.step(2), "/recipes/7/cook?step=2&servings=8&done=1,3");
        assert_eq!(place.toggle(3), "/recipes/7/cook?step=3&servings=8&done=1");
        assert!(place.is_done(3));
        assert_eq!(place.finish(), "/recipes/7?servings=8");
    }

    #[test]
    fn the_addresses_without_servings_or_done_steps_are_bare() {
        let place = Place {
            id: 7,
            servings: None,
            done: Vec::new(),
        };
        assert_eq!(place.step(1), "/recipes/7/cook?step=1");
        assert_eq!(place.finish(), "/recipes/7");
        assert_eq!(start_href(7, None), "/recipes/7/cook");
        assert_eq!(start_href(7, Some(2)), "/recipes/7/cook?servings=2");
    }
}
