//! Amounts at the start of an ingredient line: integers, decimals with a comma or dot, fractions,
//! mixed numbers, unicode fractions and ranges.

use super::words;

/// The amount a line starts with: one number, or a range from `low` to `high`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Amount {
    pub(crate) low: f64,
    pub(crate) high: Option<f64>,
}

/// The amount at the start of `text` and the text after it. `None` when `text` does not start
/// with a number.
pub(crate) fn leading(text: &str) -> Option<(Amount, &str)> {
    let (low, rest) = number(text)?;
    match range(rest) {
        Some((high, after)) => Some((
            Amount {
                low,
                high: Some(high),
            },
            after,
        )),
        None => Some((Amount { low, high: None }, rest)),
    }
}

fn range(text: &str) -> Option<(f64, &str)> {
    let text = text.trim_start();
    let after = text
        .strip_prefix(['-', '–'])
        .or_else(|| words::strip(text, "tot"))
        .or_else(|| words::strip(text, "to"))
        .or_else(|| words::strip(text, "à"))?;
    number(after.trim_start())
}

fn number(text: &str) -> Option<(f64, &str)> {
    if let Some(found) = fraction_part(text) {
        return Some(found);
    }
    let (whole, rest) = digits(text)?;
    let whole_value = finite(whole)?;
    if let Some((value, tail)) = fraction_part(rest.trim_start()) {
        return Some((whole_value + value, tail));
    }
    if let Some(found) = decimal(whole, rest) {
        return Some(found);
    }
    Some((whole_value, rest))
}

fn fraction_part(text: &str) -> Option<(f64, &str)> {
    let mut chars = text.chars();
    chars
        .next()
        .and_then(fraction)
        .map_or_else(|| quotient(text), |value| Some((value, chars.as_str())))
}

fn quotient(text: &str) -> Option<(f64, &str)> {
    let (numerator, rest) = digits(text)?;
    let (denominator, tail) = digits(rest.strip_prefix('/')?)?;
    let value = finite(numerator)? / finite(denominator)?;
    value.is_finite().then_some((value, tail))
}

fn decimal<'a>(whole: &str, rest: &'a str) -> Option<(f64, &'a str)> {
    let (decimals, tail) = digits(rest.strip_prefix(['.', ','])?)?;
    Some((finite(&format!("{whole}.{decimals}"))?, tail))
}

fn digits(text: &str) -> Option<(&str, &str)> {
    let end = text
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(text.len());
    let (digits, rest) = text.split_at_checked(end)?;
    (!digits.is_empty()).then_some((digits, rest))
}

fn finite(text: &str) -> Option<f64> {
    text.parse::<f64>().ok().filter(|value| value.is_finite())
}

const fn fraction(c: char) -> Option<f64> {
    match c {
        '½' => Some(0.5),
        '¼' => Some(0.25),
        '¾' => Some(0.75),
        '⅓' => Some(1.0 / 3.0),
        '⅔' => Some(2.0 / 3.0),
        '⅛' => Some(0.125),
        _ => None,
    }
}
