use crate::shared;

use std::ops::Range;

use bmo_text::english::*;
use serde_json::{Value, json};

use shared::text;

// A list of stretches read from the package's shape, each a start and an end.
fn stretches_of(value: &Value) -> Vec<Range<usize>> {
  let index = |stretch: &Value, key: &str| {
    usize::try_from(stretch[key].as_u64().expect("an index")).expect("an index")
  };

  value
    .as_array()
    .expect("a list of stretches")
    .iter()
    .map(|stretch| index(stretch, "start")..index(stretch, "end"))
    .collect()
}

// A letter found, in the package's shape.
fn letter_json(one: &NonEnglishLetter) -> Value {
  json!({
    "start": one.start,
    "end": one.end,
    "letter": one.letter.to_string(),
    "escaped": one.escaped,
    "place": one.place.code(),
  })
}

#[test]
fn the_shared_cases_hold() {
  shared::hold_all(
    "english",
    &[
      ("passes", &|args| {
        let weight = usize::try_from(args[1].as_u64().expect("a weight")).expect("a weight");

        Value::from(EnglishTest::new(text(&args[0]), weight).passes(text(&args[2])))
      }),
      ("holdsNonEnglish", &|args| Value::from(holds_non_english(text(&args[0])))),
      ("nonEnglishLetters", &|args| {
        let stretches = CodeStretches {
          comments: stretches_of(&args[1]["comments"]),
          literals: stretches_of(&args[1]["literals"]),
        };

        non_english_letters(text(&args[0]), &stretches).iter().map(letter_json).collect()
      }),
    ],
  );
}

#[test]
fn a_pattern_that_does_not_compile_is_refused_and_lookaround_compiles() {
  assert!(EnglishTest::try_new("(unclosed", 0).is_err());
  assert!(EnglishTest::try_new(r"(?<=x)y", 0).expect("a lookbehind compiles").passes("xy"));
}

#[test]
fn the_default_is_the_strictest_test() {
  assert!(EnglishTest::default().passes("plain English"));
  assert!(!EnglishTest::default().passes("almost English 가"));
}
