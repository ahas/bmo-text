use wasm_bindgen::prelude::*;

use super::{count, number};
use crate::translation_parts::{TRANSLATION_PART_TOKENS, shown_part, translation_parts};

#[wasm_bindgen(js_name = translationPartTokens)]
pub fn translation_part_tokens() -> f64 {
  number(TRANSLATION_PART_TOKENS)
}

#[wasm_bindgen(js_name = translationParts)]
pub fn translation_parts_of(text: &str, budget: Option<f64>) -> Vec<String> {
  translation_parts(text, budget.map_or(TRANSLATION_PART_TOKENS, count))
}

#[wasm_bindgen(js_name = shownPart)]
pub fn shown_part_of(source: &str, text: Option<String>) -> String {
  shown_part(source, text.as_deref())
}
