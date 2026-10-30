use std::collections::HashMap;
use std::sync::OnceLock;

use super::Language;

/// The text of the app in one language, keyed by dotted message keys such as `app.title`.
#[derive(Debug)]
pub(crate) struct Messages(HashMap<String, String>);

impl Messages {
    /// The messages of `language`, parsed from its embedded JSON file on first use.
    pub(crate) fn for_language(language: Language) -> &'static Self {
        static EN: OnceLock<Messages> = OnceLock::new();
        static NL: OnceLock<Messages> = OnceLock::new();
        match language {
            Language::En => {
                EN.get_or_init(|| Self::from_json(include_str!("../../locales/en.json")))
            }
            Language::Nl => {
                NL.get_or_init(|| Self::from_json(include_str!("../../locales/nl.json")))
            }
        }
    }

    /// Parses a flat JSON object of message keys to text. The locale files are checked by tests, so
    /// a file that does not parse leaves the messages empty and the keys show instead of text.
    pub(crate) fn from_json(json: &str) -> Self {
        Self(serde_json::from_str(json).unwrap_or_default())
    }

    /// The text of `key`, or the key itself when the message is missing.
    pub(crate) fn get<'a>(&'a self, key: &'a str) -> &'a str {
        self.0.get(key).map_or(key, String::as_str)
    }

    /// The text of `key` with each `{name}` replaced by the value paired with `name` in `args`.
    /// Placeholders without a matching argument are kept as they are.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "no visible text takes arguments yet; the first one that does uses it"
        )
    )]
    pub(crate) fn format(&self, key: &str, args: &[(&str, &str)]) -> String {
        let mut out = String::new();
        let mut rest = self.get(key);
        while let Some((before, after)) = rest.split_once('{') {
            out.push_str(before);
            rest = after;
            let Some((name, tail)) = rest.split_once('}') else {
                out.push('{');
                break;
            };
            if let Some((_, value)) = args.iter().find(|(arg, _)| *arg == name) {
                out.push_str(value);
            } else {
                out.push('{');
                out.push_str(name);
                out.push('}');
            }
            rest = tail;
        }
        out.push_str(rest);
        out
    }
}
