use crabbage_import::ai::{AiDraft, AiError, AiLine, AiStep, parse};
use serde_json::{Value, json};

use crate::VALID;

#[expect(clippy::expect_used, reason = "the fixture is valid JSON")]
fn edited(edit: impl FnOnce(&mut Value)) -> String {
    let mut value: Value = serde_json::from_str(VALID).expect("fixture");
    edit(&mut value);
    value.to_string()
}

#[test]
fn a_valid_draft_is_read_in_full() {
    let expected = AiDraft {
        title: "Lemon Pancakes".to_owned(),
        servings: Some(2),
        lines: vec![
            AiLine {
                amount: Some(200.0),
                unit: Some("g".to_owned()),
                name: "flour".to_owned(),
                note: None,
                ingredient_id: None,
            },
            AiLine {
                amount: Some(1.0),
                unit: Some("piece".to_owned()),
                name: "egg".to_owned(),
                note: None,
                ingredient_id: None,
            },
        ],
        steps: vec![AiStep {
            text: "Whisk flour and egg.".to_owned(),
            lines: vec![0, 1],
        }],
        prep_minutes: Some(5),
        cook_minutes: Some(10),
        photo: None,
        categories: Vec::new(),
    };
    assert_eq!(parse(VALID), Ok(expected));
}

#[test]
fn fenced_content_is_read() {
    for content in [
        format!("```json\n{VALID}\n```"),
        format!("```\n{VALID}\n```"),
        format!("  ```json\n{VALID}\n```  \n"),
    ] {
        let draft = parse(&content).expect("draft");
        assert_eq!(draft.title, "Lemon Pancakes", "{content}");
    }
}

#[test]
fn missing_optional_fields_are_empty() {
    let draft =
        parse(r#"{"title":"Toast","lines":[{"name":"bread"}],"steps":[{"text":"Toast it."}]}"#)
            .expect("draft");
    assert_eq!(draft.servings, None);
    assert_eq!(draft.prep_minutes, None);
    assert_eq!(draft.lines[0].amount, None);
    assert_eq!(draft.lines[0].unit, None);
    assert_eq!(draft.steps[0].lines, Vec::<usize>::new());
    assert_eq!(draft.photo, None);
}

#[test]
fn photo_crop_is_parsed_and_null_is_empty() {
    let content = edited(|value| {
        value["photo"] = json!({"page":1,"left":0.1,"top":0.2,"width":0.3,"height":0.4});
    });
    assert_eq!(
        parse(&content).expect("crop").photo,
        Some(crabbage_import::ai::AiPhoto {
            page: 1,
            left: 0.1,
            top: 0.2,
            width: 0.3,
            height: 0.4
        })
    );
    let content = edited(|value| value["photo"] = Value::Null);
    assert_eq!(parse(&content).expect("null photo").photo, None);
}

#[test]
fn invalid_drafts_are_refused_with_the_reason() {
    let cases = [
        ("not json".to_owned(), "not the expected JSON"),
        (
            edited(|v| v["title"] = Value::Null),
            "not the expected JSON",
        ),
        (edited(|v| v["title"] = json!("  ")), "the title is blank"),
        (edited(|v| v["lines"] = json!([])), "no ingredient lines"),
        (edited(|v| v["steps"] = json!([])), "the draft has no steps"),
        (
            edited(|v| {
                v["photo"] = json!({"page":0,"left":-0.1,"top":0.0,"width":0.5,"height":0.5});
            }),
            "photo crop is outside",
        ),
        (
            edited(|v| v["lines"][0]["unit"] = json!("handful")),
            "unknown unit: handful",
        ),
        (
            edited(|v| v["lines"][0]["amount"] = json!(-2)),
            "ingredient 1 has an invalid amount",
        ),
        (
            edited(|v| v["steps"][0]["lines"] = json!([7])),
            "step 1 refers to ingredient 8, which does not exist",
        ),
        (
            edited(|v| v["steps"][0]["text"] = json!(" ")),
            "step 1 has no text",
        ),
        (
            edited(|v| v["lines"][1]["name"] = json!("")),
            "ingredient 2 has no name",
        ),
    ];
    for (content, reason) in cases {
        match parse(&content) {
            Err(AiError::Invalid(text)) => assert!(text.contains(reason), "{text} ({reason})"),
            other => panic!("{content}: {other:?}"),
        }
    }
}

#[test]
fn units_the_line_parser_knows_are_accepted() {
    for unit in ["cup", "Cups", "oz", "fl oz", "tbsp", "el", "dl"] {
        let content = edited(|v| v["lines"][0]["unit"] = json!(unit));
        assert!(parse(&content).is_ok(), "{unit}");
    }
}
