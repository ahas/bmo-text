use serde_json::json;
use wasm_bindgen::prelude::*;

use super::{answer, number};
use crate::dashes::{read_dashes, without_dashes};

#[wasm_bindgen(js_name = withoutDashes)]
pub fn without_dashes_of(text: &str, wrapped: bool) -> String {
  without_dashes(text, wrapped)
}

#[wasm_bindgen(js_name = readDashes)]
pub fn read_dashes_of(text: &str, wrapped: bool) -> JsValue {
  let read = read_dashes(text, wrapped);

  answer(&json!({ "text": read.text, "found": number(read.found) }))
}
