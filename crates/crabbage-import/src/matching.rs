//! Matches free-text ingredient names to catalogue ingredients by name or alias.

/// A catalogue ingredient to match against.
#[derive(Debug, Clone, Copy)]
pub struct Candidate<'a> {
    /// The ingredient's id.
    pub id: i64,
    /// The ingredient's name.
    pub name: &'a str,
    /// The ingredient's alternative names.
    pub aliases: &'a [String],
}

/// The outcome of matching a list of names: one link per name, in order, and the names that found nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Matches {
    /// The matched ingredient id for each input name, `None` when nothing matched.
    pub linked: Vec<Option<i64>>,
    /// The original spelling of each unmatched name, once per normalised name, in first-seen order.
    pub unmatched: Vec<String>,
}

// NOTE: shortcut: Latin-1 accents only, use unicode-normalization if other scripts appear
const FOLDS: &[(char, char)] = &[
    ('à', 'a'),
    ('á', 'a'),
    ('â', 'a'),
    ('ã', 'a'),
    ('ä', 'a'),
    ('å', 'a'),
    ('ç', 'c'),
    ('è', 'e'),
    ('é', 'e'),
    ('ê', 'e'),
    ('ë', 'e'),
    ('ì', 'i'),
    ('í', 'i'),
    ('î', 'i'),
    ('ï', 'i'),
    ('ñ', 'n'),
    ('ò', 'o'),
    ('ó', 'o'),
    ('ô', 'o'),
    ('õ', 'o'),
    ('ö', 'o'),
    ('ù', 'u'),
    ('ú', 'u'),
    ('û', 'u'),
    ('ü', 'u'),
    ('ý', 'y'),
    ('ÿ', 'y'),
];

/// The name lowercased, trimmed, with internal whitespace collapsed to one space and diacritics removed.
#[must_use]
pub fn normalise(name: &str) -> String {
    name.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .map(|c| {
            FOLDS
                .iter()
                .find(|(from, _)| *from == c)
                .map_or(c, |(_, to)| *to)
        })
        .collect()
}

/// The id of the first candidate whose name equals `name` once normalised, otherwise the first whose
/// alias does. Empty input matches nothing.
#[must_use]
pub fn match_ingredient(name: &str, candidates: &[Candidate<'_>]) -> Option<i64> {
    let wanted = normalise(name);
    if wanted.is_empty() {
        return None;
    }
    candidates
        .iter()
        .find(|c| normalise(c.name) == wanted)
        .or_else(|| {
            candidates
                .iter()
                .find(|c| c.aliases.iter().any(|alias| normalise(alias) == wanted))
        })
        .map(|c| c.id)
}

/// Links each name with `match_ingredient`, and collects the names that found nothing.
#[must_use]
pub fn match_all(names: &[&str], candidates: &[Candidate<'_>]) -> Matches {
    let mut linked = Vec::with_capacity(names.len());
    let mut unmatched: Vec<String> = Vec::new();
    for name in names {
        let found = match_ingredient(name, candidates);
        if found.is_none() {
            let key = normalise(name);
            if !unmatched.iter().any(|seen| normalise(seen) == key) {
                unmatched.push((*name).to_owned());
            }
        }
        linked.push(found);
    }
    Matches { linked, unmatched }
}
