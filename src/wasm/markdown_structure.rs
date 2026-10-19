use serde_json::{Value, json};
use wasm_bindgen::prelude::*;

use super::answer;
use crate::markdown_structure::{
  halve_text, heading_slug, markdown_headings, markdown_sections, plain_heading,
};

#[wasm_bindgen(js_name = markdownHeadings)]
pub fn markdown_headings_of(source: &str) -> JsValue {
  let headings: Vec<Value> = markdown_headings(source)
    .iter()
    .map(|heading| json!({ "depth": heading.depth, "text": heading.text, "slug": heading.slug, "line": heading.line }))
    .collect();

  answer(&Value::from(headings))
}

#[wasm_bindgen(js_name = plainHeading)]
pub fn plain_heading_of(text: &str) -> String {
  plain_heading(text)
}

#[wasm_bindgen(js_name = headingSlug)]
pub fn heading_slug_of(text: &str) -> String {
  heading_slug(text)
}

#[wasm_bindgen(js_name = markdownSections)]
pub fn markdown_sections_of(source: &str) -> Vec<String> {
  markdown_sections(source)
}

#[wasm_bindgen(js_name = halveText)]
pub fn halve_text_of(text: &str) -> JsValue {
  halve_text(text).map_or(JsValue::NULL, |cut| {
    answer(&json!({ "halves": [cut.halves.0, cut.halves.1], "joint": cut.joint }))
  })
}
