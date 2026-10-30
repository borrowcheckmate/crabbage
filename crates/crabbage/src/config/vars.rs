use url::Url;

use super::error::Problem;
use super::settings::{AiConfig, Language};

pub(super) struct Vars<F> {
    lookup: F,
    pub(super) problems: Vec<Problem>,
}

impl<F: Fn(&str) -> Option<String>> Vars<F> {
    pub(super) const fn new(lookup: F) -> Self {
        Self {
            lookup,
            problems: Vec::new(),
        }
    }

    pub(super) fn get(&self, name: &str) -> Option<String> {
        (self.lookup)(name).filter(|value| !value.is_empty())
    }

    pub(super) fn required(&mut self, name: &'static str) -> Option<String> {
        let value = self.get(name);
        if value.is_none() {
            self.problems.push(Problem::Missing(name));
        }
        value
    }

    pub(super) fn url(&mut self, name: &'static str) -> Option<Url> {
        let value = self.required(name)?;
        self.parse_url(name, &value)
    }

    fn parse_url(&mut self, name: &'static str, value: &str) -> Option<Url> {
        let reason = match Url::parse(value) {
            Ok(url) if matches!(url.scheme(), "http" | "https") => return Some(url),
            Ok(url) => format!("scheme must be http or https, not {}", url.scheme()),
            Err(error) => error.to_string(),
        };
        self.problems.push(Problem::Invalid { name, reason });
        None
    }

    pub(super) fn language(&mut self) -> Option<Language> {
        match self.get("CRABBAGE_LANGUAGE").as_deref() {
            None | Some("en") => Some(Language::En),
            Some("nl") => Some(Language::Nl),
            Some(other) => {
                self.problems.push(Problem::Invalid {
                    name: "CRABBAGE_LANGUAGE",
                    reason: format!("expected en or nl, not {other}"),
                });
                None
            }
        }
    }

    /// Returns `None` when AI is disabled and when its settings are invalid; the problems say which.
    pub(super) fn ai(&mut self) -> Option<AiConfig> {
        match (self.get("AI_BASE_URL"), self.get("AI_MODEL")) {
            (None, None) => None,
            (Some(base_url), Some(model)) => {
                let base_url = self.parse_url("AI_BASE_URL", &base_url)?;
                Some(AiConfig {
                    base_url,
                    api_key: self.get("AI_API_KEY"),
                    model,
                })
            }
            (Some(_), None) => {
                self.problems.push(Problem::Missing("AI_MODEL"));
                None
            }
            (None, Some(_)) => {
                self.problems.push(Problem::Missing("AI_BASE_URL"));
                None
            }
        }
    }
}
