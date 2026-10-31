//! Ingredient name matching: normalisation, names and aliases, and unmatched names.

use crabbage_import::lines::parse_line;
use crabbage_import::matching::{Candidate, match_all, match_ingredient, normalise};

fn aliases(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_owned()).collect()
}

#[test]
fn case_is_ignored() {
    assert_eq!(normalise("FLOUR"), "flour");
    let flour = [Candidate {
        id: 1,
        name: "Flour",
        aliases: &[],
    }];
    assert_eq!(match_ingredient("FLOUR", &flour), Some(1));
}

#[test]
fn whitespace_is_trimmed_and_collapsed() {
    assert_eq!(normalise("  brown   sugar "), "brown sugar");
    let sugar = [Candidate {
        id: 2,
        name: "Brown sugar",
        aliases: &[],
    }];
    assert_eq!(match_ingredient("  brown   sugar ", &sugar), Some(2));
}

#[test]
fn diacritics_are_removed() {
    assert_eq!(normalise("crème fraîche"), "creme fraiche");
    assert_eq!(normalise("CRÈME FRAÎCHE"), "creme fraiche");
    assert_eq!(normalise("jalapeño"), "jalapeno");
    let cream = [Candidate {
        id: 3,
        name: "crème fraîche",
        aliases: &[],
    }];
    assert_eq!(match_ingredient("creme fraiche", &cream), Some(3));
    let pepper = [Candidate {
        id: 4,
        name: "Jalapeno",
        aliases: &[],
    }];
    assert_eq!(match_ingredient("jalapeño", &pepper), Some(4));
}

#[test]
fn exact_name_links() {
    let candidates = [
        Candidate {
            id: 1,
            name: "butter",
            aliases: &[],
        },
        Candidate {
            id: 2,
            name: "flour",
            aliases: &[],
        },
    ];
    assert_eq!(match_ingredient("flour", &candidates), Some(2));
}

#[test]
fn alias_links() {
    let plain = aliases(&["all-purpose flour", "plain flour"]);
    let candidates = [Candidate {
        id: 7,
        name: "flour",
        aliases: &plain,
    }];
    assert_eq!(match_ingredient("Plain  Flour", &candidates), Some(7));
}

#[test]
fn name_beats_alias_of_another_candidate() {
    let alias = aliases(&["cilantro"]);
    let candidates = [
        Candidate {
            id: 1,
            name: "coriander",
            aliases: &alias,
        },
        Candidate {
            id: 2,
            name: "cilantro",
            aliases: &[],
        },
    ];
    assert_eq!(match_ingredient("cilantro", &candidates), Some(2));
}

#[test]
fn partial_names_do_not_match() {
    let candidates = [Candidate {
        id: 1,
        name: "rice flour",
        aliases: &[],
    }];
    assert_eq!(match_ingredient("flour", &candidates), None);
    assert_eq!(match_ingredient("rice", &candidates), None);
}

#[test]
fn empty_input_matches_nothing() {
    let candidates = [Candidate {
        id: 1,
        name: "",
        aliases: &[],
    }];
    assert_eq!(match_ingredient("", &candidates), None);
    assert_eq!(match_ingredient("   ", &candidates), None);
}

#[test]
fn unmatched_names_are_reported_once_in_order() {
    let candidates = [Candidate {
        id: 1,
        name: "flour",
        aliases: &[],
    }];
    let matches = match_all(
        &["Salt", "FLOUR", "pepper", " salt ", "Pepper"],
        &candidates,
    );
    assert_eq!(matches.linked, [None, Some(1), None, None, None]);
    assert_eq!(matches.unmatched, ["Salt", "pepper"]);
}

#[test]
fn recipe_lines_match_common_english_and_dutch_catalog_aliases() {
    let onion_aliases = aliases(&["onions", "gele ui", "gele uien"]);
    let tomato_aliases = aliases(&["tomatoes", "tomaten"]);
    let candidates = [
        Candidate {
            id: 1,
            name: "Onion",
            aliases: &onion_aliases,
        },
        Candidate {
            id: 2,
            name: "Tomato",
            aliases: &tomato_aliases,
        },
    ];
    for (line, expected) in [
        ("2 onions, diced", 1),
        ("1 gele ui", 1),
        ("3 tomatoes", 2),
        ("4 tomaten, gehalveerd", 2),
    ] {
        let parsed = parse_line(line);
        assert!(parsed.parsed, "{line}");
        assert_eq!(
            match_ingredient(&parsed.name, &candidates),
            Some(expected),
            "{line}"
        );
    }
}
