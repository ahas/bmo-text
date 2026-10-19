use serde_json::{Value, json};
use wasm_bindgen::prelude::*;

use super::{answer, character_at, units_at};
use crate::js::chars;
use crate::web_address::{
  address_as_written, bare_address_at, glued_after_address, is_web_address, markdown_link_at, web_addresses,
};

#[wasm_bindgen(js_name = addressAsWritten)]
pub fn address_as_written_of(candidate: &str) -> String {
  address_as_written(candidate)
}

#[wasm_bindgen(js_name = isWebAddress)]
pub fn is_web_address_of(url: &str) -> bool {
  is_web_address(url)
}

#[wasm_bindgen(js_name = markdownLinkAt)]
pub fn markdown_link_at_of(text: &str, at: f64) -> JsValue {
  let characters = chars(text);

  markdown_link_at(&characters, character_at(&characters, at)).map_or(JsValue::NULL, |link| {
    answer(&json!({ "label": link.label, "url": link.url, "end": units_at(&characters, link.end) }))
  })
}

#[wasm_bindgen(js_name = bareAddressAt)]
pub fn bare_address_at_of(text: &str, at: f64) -> Option<String> {
  let characters = chars(text);

  bare_address_at(&characters, character_at(&characters, at))
}

#[wasm_bindgen(js_name = gluedAfterAddress)]
pub fn glued_after_address_of(text: &str, end: f64) -> bool {
  let characters = chars(text);

  glued_after_address(&characters, character_at(&characters, end))
}

#[wasm_bindgen(js_name = webAddresses)]
pub fn web_addresses_of(text: &str) -> JsValue {
  let characters = chars(text);
  let found: Vec<Value> = web_addresses(&characters)
    .into_iter()
    .map(|found| {
      json!({
        "start": units_at(&characters, found.start),
        "end": units_at(&characters, found.end),
        "url": found.url,
        "label": found.label,
      })
    })
    .collect();

  answer(&Value::from(found))
}
