use serde_json::{Map, Value, json};
use wasm_bindgen::prelude::*;

use super::{answer, argument, count};
use crate::finding_markdown::{
  FindingWords, PART_KEYS, finding_excerpt, finding_markdown, footnote_marks, picture_caption,
};

#[wasm_bindgen(js_name = partKeys)]
pub fn part_keys() -> Vec<String> {
  PART_KEYS.iter().map(|key| (*key).to_owned()).collect()
}

// The words in the package's shape, each label under its camelCase key.
fn words_json(words: &FindingWords) -> Value {
  let parts: Map<String, Value> = words
    .parts
    .iter()
    .map(|(key, word)| (key.clone(), Value::from(word.as_str())))
    .collect();

  json!({
    "parts": parts,
    "address": words.address,
    "map": words.map,
    "streetView": words.street_view,
    "released": words.released,
    "newer": words.newer,
    "productPage": words.product_page,
    "stock": words.stock,
    "usual": words.usual,
    "deal": words.deal,
    "review": words.review,
    "noRating": words.no_rating,
    "menu": words.menu,
    "dish": words.dish,
    "price": words.price,
    "notes": words.notes,
  })
}

// The words read from the package's shape, a label it leaves out being the default's.
fn words_of(value: &Value) -> FindingWords {
  let default = FindingWords::default();
  let word = |key: &str, otherwise: String| value[key].as_str().map_or(otherwise, str::to_owned);

  FindingWords {
    parts: value["parts"]
      .as_object()
      .map(|parts| {
        parts
          .iter()
          .filter_map(|(key, word)| word.as_str().map(|word| (key.clone(), word.to_owned())))
          .collect()
      })
      .unwrap_or_default(),
    address: word("address", default.address),
    map: word("map", default.map),
    street_view: word("streetView", default.street_view),
    released: word("released", default.released),
    newer: word("newer", default.newer),
    product_page: word("productPage", default.product_page),
    stock: word("stock", default.stock),
    usual: word("usual", default.usual),
    deal: word("deal", default.deal),
    review: word("review", default.review),
    no_rating: word("noRating", default.no_rating),
    menu: word("menu", default.menu),
    dish: word("dish", default.dish),
    price: word("price", default.price),
    notes: word("notes", default.notes),
  }
}

#[wasm_bindgen(js_name = englishFindingWords)]
pub fn english_finding_words() -> JsValue {
  answer(&words_json(&FindingWords::default()))
}

#[wasm_bindgen(js_name = findingMarkdown)]
pub fn finding_markdown_of(text: &str, words: &str) -> Result<String, JsError> {
  Ok(finding_markdown(text, &words_of(&argument(words)?)))
}

#[wasm_bindgen(js_name = footnoteMarks)]
pub fn footnote_marks_of(line: &str) -> String {
  footnote_marks(line)
}

#[wasm_bindgen(js_name = pictureCaption)]
pub fn picture_caption_of(captioned: &str) -> String {
  picture_caption(captioned)
}

#[wasm_bindgen(js_name = findingExcerpt)]
pub fn finding_excerpt_of(texts: Vec<String>, limit: f64) -> String {
  finding_excerpt(&texts.iter().map(String::as_str).collect::<Vec<_>>(), count(limit))
}
