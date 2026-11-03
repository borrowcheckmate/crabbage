//! Exact step durations in localized minutes and seconds.

use crate::i18n::t;

pub(crate) fn duration_text(seconds: i64) -> String {
    let seconds = seconds.max(0);
    let (minutes, rest) = (seconds / 60, seconds % 60);
    if minutes == 0 {
        format!("{rest} {}", t("unit.seconds"))
    } else if rest == 0 {
        format!("{minutes} {}", t("unit.minutes"))
    } else {
        format!(
            "{minutes} {} {rest} {}",
            t("unit.minutes"),
            t("unit.seconds")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::duration_text;
    use crate::i18n::Language;
    use leptos::prelude::{Owner, provide_context};

    #[test]
    fn durations_keep_seconds_in_both_languages() {
        for language in [Language::En, Language::Nl] {
            Owner::new().with(|| {
                provide_context(language);
                for (seconds, expected) in [
                    (0, "0 sec"),
                    (10, "10 sec"),
                    (59, "59 sec"),
                    (60, "1 min"),
                    (70, "1 min 10 sec"),
                    (3600, "60 min"),
                ] {
                    assert_eq!(duration_text(seconds), expected);
                }
            });
        }
    }
}
