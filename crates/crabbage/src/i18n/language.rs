/// The language of the app's text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    /// English.
    En,
    /// Dutch.
    Nl,
}

impl Language {
    /// The code of the language, as set in `CRABBAGE_LANGUAGE` and in the `lang` attribute of the page.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Nl => "nl",
        }
    }

    /// The language with the given code, or `None` when the app does not speak it.
    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "en" => Some(Self::En),
            "nl" => Some(Self::Nl),
            _ => None,
        }
    }
}
