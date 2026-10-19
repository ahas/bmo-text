use std::ops::Range;

use serde_json::{Value, json};
use wasm_bindgen::prelude::*;

use super::{answer, argument, count, list};
use crate::english::{self, CodeStretches, holds_non_english, non_english_letters};

/// The crate's `EnglishTest`, held for the package's class of the same name.
#[wasm_bindgen]
pub struct EnglishTest(english::EnglishTest);

#[wasm_bindgen]
impl EnglishTest {
  /// The test `try_new` makes, throwing the compiler's message for a pattern that does not compile.
  #[wasm_bindgen(constructor)]
  pub fn new(set_aside: &str, latin_per_hangul: f64) -> Result<EnglishTest, JsError> {
    english::EnglishTest::try_new(set_aside, count(latin_per_hangul))
      .map(Self)
      .map_err(|error| JsError::new(&error))
  }

  pub fn passes(&self, text: &str) -> bool {
    self.0.passes(text)
  }
}

#[wasm_bindgen(js_name = holdsNonEnglish)]
pub fn holds_non_english_of(text: &str) -> bool {
  holds_non_english(text)
}

// A stretch in the package's shape, its start and its end.
fn stretch_of(value: &Value) -> Result<Range<usize>, JsError> {
  let index = |key: &str| {
    value[key]
      .as_f64()
      .map(count)
      .ok_or_else(|| JsError::new(&format!("The field {key} of a stretch is not a number.")))
  };

  Ok(index("start")?..index("end")?)
}

/// The letters of a text that are not English, placed by the stretches the package wrote as JSON.
#[wasm_bindgen(js_name = nonEnglishLetters)]
pub fn non_english_letters_of(text: &str, stretches: &str) -> Result<JsValue, JsError> {
  let stretches = argument(stretches)?;
  let stretches = CodeStretches {
    comments: list(&stretches["comments"], stretch_of)?,
    literals: list(&stretches["literals"], stretch_of)?,
  };
  let letters: Vec<Value> = non_english_letters(text, &stretches)
    .into_iter()
    .map(|one| {
      json!({
        "start": one.start,
        "end": one.end,
        "letter": one.letter.to_string(),
        "escaped": one.escaped,
        "place": one.place.code(),
      })
    })
    .collect();

  Ok(answer(&Value::from(letters)))
}
