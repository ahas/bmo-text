//! The answer ceiling held to the cases the package's is held to, in `cases/answer-cap.json`: a piece of
//! text is a string or a text and how many times it repeats, and a case's items are its pieces taken
//! `copies` times over.

use std::fs;
use std::path::Path;

use bmo_text::answer_cap::{MAX_ANSWER_CHARS, MAX_VALUE_CHARS, Said, cap_items, cap_value};
use serde_json::Value;

fn cases() -> Value {
  let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("cases/answer-cap.json");

  serde_json::from_str(&fs::read_to_string(path).expect("the cases file reads"))
    .expect("the cases file is JSON")
}

fn piece(value: &Value) -> String {
  match value {
    Value::String(text) => text.clone(),
    repeated => repeated["text"].as_str().expect("a repeated piece's text").repeat(
      usize::try_from(repeated["times"].as_u64().expect("how many times a piece repeats"))
        .expect("a count"),
    ),
  }
}

fn joined(pieces: &Value) -> String {
  pieces.as_array().expect("a list of pieces").iter().map(piece).collect()
}

fn count(value: &Value) -> Option<usize> {
  value.as_u64().map(|count| usize::try_from(count).expect("a count"))
}

fn lines(kept: &[String]) -> String {
  kept.join("\n")
}

fn json(kept: &[String]) -> String {
  serde_json::to_string_pretty(kept).expect("a list of strings serialises")
}

#[test]
fn a_list_is_cut_as_the_package_cuts_it() {
  for case in cases()["items"].as_array().expect("the list cases") {
    let name = case["name"].as_str().expect("a case is named");
    let pieces: Vec<String> = case["items"].as_array().expect("a case's items").iter().map(piece).collect();
    let items: Vec<String> = pieces
      .iter()
      .cycle()
      .take(pieces.len() * count(&case["copies"]).unwrap_or(1))
      .cloned()
      .collect();
    let said = Said {
      limit: count(&case["limit"]).unwrap_or(MAX_ANSWER_CHARS),
      ..Said::new(case["unit"].as_str().expect("a unit"), case["how"].as_str().expect("a how"))
    };
    let capped = match case["render"].as_str() {
      Some("lines") => cap_items(&items, lines, &said),
      Some("json") => cap_items(&items, json, &said),
      other => panic!("{name}: no render {other:?}"),
    };

    assert_eq!(capped.kept.len(), count(&case["kept"]).expect("kept"), "{name}");
    assert_eq!(capped.kept, items[..capped.kept.len()], "{name}");
    assert_eq!(capped.left, count(&case["left"]).expect("left"), "{name}");
    assert_eq!(capped.note.as_deref(), case["note"].as_str(), "{name}");
  }
}

#[test]
fn a_value_is_cut_as_the_package_cuts_it() {
  for case in cases()["values"].as_array().expect("the value cases") {
    let name = case["name"].as_str().expect("a case is named");
    let limit = count(&case["limit"]).unwrap_or(MAX_VALUE_CHARS);

    assert_eq!(cap_value(&joined(&case["text"]), limit), joined(&case["expect"]), "{name}");
  }
}
