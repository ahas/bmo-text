use wasm_bindgen::prelude::*;

use crate::answer_language::{holds_korean, text_language};

#[wasm_bindgen(js_name = holdsKorean)]
pub fn holds_korean_of(text: &str) -> bool {
  holds_korean(text)
}

#[wasm_bindgen(js_name = textLanguage)]
pub fn text_language_of(text: &str) -> String {
  text_language(text).code().to_owned()
}
