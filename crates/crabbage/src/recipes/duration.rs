//! The duration of a step as the pages show it: whole minutes, rounded up.

use crate::i18n::t;

pub(crate) fn duration_text(seconds: i64) -> String {
    format!("{} {}", (seconds + 59) / 60, t("unit.minutes"))
}
