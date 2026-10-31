//! The durations a step's text names, in English and Dutch: `25 minutes`, `1 hour 30 minutes`,
//! `10-15 minutes` (the upper bound), `1,5 uur`, `anderhalf uur` and `een half uur`.

use std::iter::Peekable;
use std::str::Chars;
use std::time::Duration;

const UNITS: [(&str, u64); 18] = [
    ("hour", 3600),
    ("hours", 3600),
    ("hr", 3600),
    ("hrs", 3600),
    ("uur", 3600),
    ("uren", 3600),
    ("minute", 60),
    ("minutes", 60),
    ("min", 60),
    ("mins", 60),
    ("minuut", 60),
    ("minuten", 60),
    ("second", 1),
    ("seconds", 1),
    ("sec", 1),
    ("secs", 1),
    ("seconde", 1),
    ("seconden", 1),
];

#[derive(Debug, PartialEq, Eq)]
enum Token {
    Number { value: u64, scale: u64 },
    Word(String),
}

struct Span {
    first: usize,
    last: usize,
    seconds: u64,
    unit: u64,
}

/// The durations named in `text`, in the order they appear. Units that get smaller one after the
/// other (`1 hour 30 minutes`, `1 uur en 30 minuten`) make one duration. Zero amounts are left out.
#[must_use]
pub fn detect_durations(text: &str) -> Vec<Duration> {
    let tokens = tokenize(text);
    let spans: Vec<Span> = tokens
        .iter()
        .enumerate()
        .filter_map(|(last, token)| match token {
            Token::Word(word) => {
                let unit = unit_seconds(word)?;
                let (first, seconds) = amount_before(&tokens, last, unit)?;
                Some(Span {
                    first,
                    last,
                    seconds,
                    unit,
                })
            }
            Token::Number { .. } => None,
        })
        .collect();
    let mut merged: Vec<Span> = Vec::new();
    for span in spans {
        match merged.last_mut() {
            Some(previous) if joins(&tokens, previous, &span) => {
                previous.seconds = previous.seconds.saturating_add(span.seconds);
                previous.last = span.last;
                previous.unit = span.unit;
            }
            _ => merged.push(span),
        }
    }
    merged
        .into_iter()
        .filter(|span| span.seconds > 0)
        .map(|span| Duration::from_secs(span.seconds))
        .collect()
}

fn tokenize(text: &str) -> Vec<Token> {
    let lower = text.to_lowercase();
    let mut chars = lower.chars().peekable();
    let mut tokens = Vec::new();
    while let Some(&next) = chars.peek() {
        if next.is_ascii_digit() {
            tokens.push(number(&mut chars));
        } else if next.is_alphabetic() {
            tokens.push(Token::Word(run(&mut chars, char::is_alphabetic)));
        } else {
            chars.next();
        }
    }
    tokens
}

fn run(chars: &mut Peekable<Chars<'_>>, keep: fn(char) -> bool) -> String {
    let mut out = String::new();
    while let Some(c) = chars.next_if(|&c| keep(c)) {
        out.push(c);
    }
    out
}

fn number(chars: &mut Peekable<Chars<'_>>) -> Token {
    let mut digits = run(chars, |c| c.is_ascii_digit());
    let mut scale = Some(1_u64);
    if chars.next_if(|&c| matches!(c, '.' | ',')).is_some() {
        let fraction = run(chars, |c| c.is_ascii_digit());
        scale = fraction
            .chars()
            .try_fold(1_u64, |acc, _| acc.checked_mul(10));
        digits.push_str(&fraction);
    }
    match (digits.parse::<u64>(), scale) {
        (Ok(value), Some(scale)) => Token::Number { value, scale },
        _ => Token::Word(String::new()),
    }
}

fn unit_seconds(word: &str) -> Option<u64> {
    UNITS
        .iter()
        .find(|(name, _)| *name == word)
        .map(|(_, seconds)| *seconds)
}

/// The amount in front of the unit at `at`, as the index it starts at and its seconds.
fn amount_before(tokens: &[Token], at: usize, unit: u64) -> Option<(usize, u64)> {
    let before = |offset: usize| {
        at.checked_sub(offset)
            .and_then(|index| tokens.get(index).map(|token| (index, token)))
    };
    let (index, previous) = before(1)?;
    match previous {
        Token::Number { value, scale } => {
            Some((index, value.checked_mul(unit)?.checked_div(*scale)?))
        }
        Token::Word(word) => match word.as_str() {
            "een" | "a" | "an" => Some((index, unit)),
            "anderhalf" | "anderhalve" => Some((index, unit.checked_mul(3)? / 2)),
            "half" => {
                let (article_index, article) = before(2)?;
                matches!(article, Token::Word(word) if is_article(word))
                    .then_some((article_index, unit / 2))
            }
            _ => None,
        },
    }
}

fn is_article(word: &str) -> bool {
    matches!(word, "een" | "a" | "an")
}

fn joins(tokens: &[Token], before: &Span, after: &Span) -> bool {
    after.unit < before.unit
        && match after.first.checked_sub(before.last) {
            Some(1) => true,
            Some(2) => matches!(
                tokens.get(before.last + 1),
                Some(Token::Word(word)) if word == "en" || word == "and"
            ),
            _ => false,
        }
}
