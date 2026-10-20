use serde_json::{Value, json};
use wasm_bindgen::prelude::*;

use super::answer;
use crate::markdown_structure::{
  MarkdownHeading, halve_text, heading_slug, markdown_headings, markdown_sections, markdown_structure,
  plain_heading,
};

fn headings_json(headings: &[MarkdownHeading]) -> Value {
  headings
    .iter()
    .map(|heading| json!({ "depth": heading.depth, "text": heading.text, "slug": heading.slug, "line": heading.line }))
    .collect()
}

#[wasm_bindgen(js_name = markdownHeadings)]
pub fn markdown_headings_of(source: &str) -> JsValue {
  answer(&headings_json(&markdown_headings(source)))
}

#[wasm_bindgen(js_name = markdownStructure)]
pub fn markdown_structure_of(source: &str) -> JsValue {
  let structure = markdown_structure(source);

  answer(&json!({ "headings": headings_json(&structure.headings), "sections": structure.sections }))
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
