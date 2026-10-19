use serde_json::{Value, json};
use wasm_bindgen::prelude::*;

use super::answer;
use crate::html_reading::read_html;

/// The reading in the package's shape, each picture's bytes the bytes themselves rather than a list of
/// numbers.
#[wasm_bindgen(js_name = readHtml)]
pub fn read_html_of(html: &str, stem: &str) -> JsValue {
  let reading = read_html(html, stem);
  let pictures: Vec<Value> = reading
    .pictures
    .iter()
    .map(|picture| json!({ "name": picture.name, "type": picture.media_type, "bytes": null }))
    .collect();
  let read = answer(&json!({
    "title": reading.title,
    "description": reading.description,
    "lines": reading.lines,
    "pictures": pictures,
  }));
  let shown = js_sys::Reflect::get(&read, &"pictures".into()).map(|shown| js_sys::Array::from(&shown));

  if let Ok(shown) = shown {
    for (at, picture) in reading.pictures.iter().enumerate() {
      let bytes = js_sys::Uint8Array::from(picture.bytes.as_slice());
      let _ = js_sys::Reflect::set(&shown.get(at as u32), &"bytes".into(), &bytes);
    }
  }

  read
}
