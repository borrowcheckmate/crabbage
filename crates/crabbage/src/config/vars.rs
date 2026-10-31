use url::Url;

use super::error::Problem;
use crate::i18n::Language;

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

    /// A `true` or `false` setting, `false` when unset.
    pub(super) fn flag(&mut self, name: &'static str) -> bool {
        match self.get(name).as_deref() {
            None | Some("false") => false,
            Some("true") => true,
            Some(other) => {
                self.problems.push(Problem::Invalid {
                    name,
                    reason: format!("expected true or false, not {other}"),
                });
                false
            }
        }
    }

    pub(super) fn language(&mut self) -> Option<Language> {
        let Some(code) = self.get("CRABBAGE_LANGUAGE") else {
            return Some(Language::En);
        };
        let language = Language::from_code(&code);
        if language.is_none() {
            self.problems.push(Problem::Invalid {
                name: "CRABBAGE_LANGUAGE",
                reason: format!("expected en or nl, not {code}"),
            });
        }
        language
    }
}
