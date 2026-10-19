use wasm_bindgen::prelude::*;

use crate::answer_language::holds_korean;

#[wasm_bindgen(js_name = holdsKorean)]
pub fn holds_korean_of(text: &str) -> bool {
  holds_korean(text)
}
