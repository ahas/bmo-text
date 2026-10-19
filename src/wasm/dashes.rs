use wasm_bindgen::prelude::*;

use crate::dashes::without_dashes;

#[wasm_bindgen(js_name = withoutDashes)]
pub fn without_dashes_of(text: &str, wrapped: bool) -> String {
  without_dashes(text, wrapped)
}
