use std::cell::RefCell;

use serde_json::json;
use wasm_bindgen::prelude::*;

use super::{answer, count, number};
use crate::answer_cap::{MAX_ANSWER_CHARS, MAX_VALUE_CHARS, Said, cap_items, cap_value};

#[wasm_bindgen(js_name = maxAnswerChars)]
pub fn max_answer_chars() -> f64 {
  number(MAX_ANSWER_CHARS)
}

#[wasm_bindgen(js_name = maxValueChars)]
pub fn max_value_chars() -> f64 {
  number(MAX_VALUE_CHARS)
}

/// The cut of `length` items, `render` being given how many from the front a prefix holds and giving what the
/// caller prints for it: how many are kept, how many are left and the note. An error `render` throws is
/// thrown on once the cut has stopped calling it.
#[wasm_bindgen(js_name = capItems)]
pub fn cap_items_of(
  length: f64,
  render: &js_sys::Function,
  unit: &str,
  how: &str,
  limit: Option<f64>,
) -> Result<JsValue, JsValue> {
  let thrown = RefCell::new(None);
  let items: Vec<usize> = (0..count(length)).collect();
  let rendered = |prefix: &[usize]| {
    if thrown.borrow().is_some() {
      return String::new();
    }

    render.call1(&JsValue::NULL, &JsValue::from(number(prefix.len()))).map_or_else(
      |error| {
        thrown.replace(Some(error));
        String::new()
      },
      |text| text.as_string().unwrap_or_default(),
    )
  };
  let said = Said {
    limit: limit.map_or(MAX_ANSWER_CHARS, count),
    ..Said::new(unit, how)
  };
  let capped = cap_items(&items, rendered, &said);

  match thrown.into_inner() {
    Some(error) => Err(error),
    None => Ok(answer(&json!({ "kept": capped.kept.len(), "left": capped.left, "note": capped.note }))),
  }
}

#[wasm_bindgen(js_name = capValue)]
pub fn cap_value_of(text: &str, limit: Option<f64>) -> String {
  cap_value(text, limit.map_or(MAX_VALUE_CHARS, count))
}
