use serde_json::{Value, json};
use wasm_bindgen::prelude::*;

use super::{answer, argument, number, string};
use crate::format::{
  LONG_TEXT_LENGTH, MarkupFault, escape_attribute, escape_html, expandable, expandable_markup,
  fault_reason, is_linkable, markup_fault, visible_text,
};

#[wasm_bindgen(js_name = escapeHtml)]
pub fn escape_html_of(text: &str) -> String {
  escape_html(text)
}

#[wasm_bindgen(js_name = escapeAttribute)]
pub fn escape_attribute_of(value: &str) -> String {
  escape_attribute(value)
}

#[wasm_bindgen(js_name = isLinkable)]
pub fn is_linkable_of(url: &str) -> bool {
  is_linkable(url)
}

#[wasm_bindgen(js_name = visibleText)]
pub fn visible_text_of(markup: &str) -> String {
  visible_text(markup)
}

#[wasm_bindgen(js_name = expandableMarkup)]
pub fn expandable_markup_of(markup: &str) -> String {
  expandable_markup(markup)
}

#[wasm_bindgen(js_name = expandable)]
pub fn expandable_of(text: &str) -> String {
  expandable(text)
}

#[wasm_bindgen(js_name = longTextLength)]
pub fn long_text_length() -> f64 {
  number(LONG_TEXT_LENGTH)
}

// A fault in the package's shape, a tagged union by its kind.
fn fault_json(fault: &MarkupFault) -> Value {
  match fault {
    MarkupFault::Unescaped { character } => {
      json!({ "kind": "unescaped", "character": character.to_string() })
    }
    MarkupFault::Unsupported { tag } => json!({ "kind": "unsupported", "tag": tag }),
    MarkupFault::Unclosed { tag } => json!({ "kind": "unclosed", "tag": tag }),
    MarkupFault::Unopened { tag, open } => json!({ "kind": "unopened", "tag": tag, "open": open }),
    MarkupFault::Nested { tag, inside } => json!({ "kind": "nested", "tag": tag, "inside": inside }),
  }
}

// A fault read from the package's shape.
fn fault_of(value: &Value) -> Result<MarkupFault, JsError> {
  Ok(match string(value, "kind")?.as_str() {
    "unescaped" => MarkupFault::Unescaped {
      character: string(value, "character")?.chars().next().unwrap_or_default(),
    },
    "unsupported" => MarkupFault::Unsupported {
      tag: string(value, "tag")?,
    },
    "unclosed" => MarkupFault::Unclosed {
      tag: string(value, "tag")?,
    },
    "unopened" => MarkupFault::Unopened {
      tag: string(value, "tag")?,
      open: value["open"].as_str().map(str::to_owned),
    },
    "nested" => MarkupFault::Nested {
      tag: string(value, "tag")?,
      inside: string(value, "inside")?,
    },
    other => return Err(JsError::new(&format!("No markup fault is of the kind {other}."))),
  })
}

#[wasm_bindgen(js_name = markupFault)]
pub fn markup_fault_of(markup: &str) -> JsValue {
  markup_fault(markup).as_ref().map_or(JsValue::NULL, |fault| answer(&fault_json(fault)))
}

#[wasm_bindgen(js_name = faultReason)]
pub fn fault_reason_of(fault: &str) -> Result<String, JsError> {
  Ok(fault_reason(&fault_of(&argument(fault)?)?))
}
