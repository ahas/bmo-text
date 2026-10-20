//! The binding the package's WebAssembly build exports (docs/design.md, 「One implementation」), compiled only
//! with the `wasm` feature. Each export takes the package's arguments, calls the crate's public functions and
//! writes the answer in the package's shape: camelCase keys, `null` for none, every index and length in
//! UTF-16 units. A structured argument arrives as the JSON the package's module wrote of it, and a structured
//! answer leaves as the object it is. No reading is made here.

mod answer_cap;
mod answer_language;
mod dashes;
mod english;
mod finding_markdown;
mod format;
mod html_reading;
mod markdown;
mod markdown_structure;
mod translation_parts;
mod web_address;
mod wording;

use serde_json::Value;
use wasm_bindgen::prelude::*;

use crate::js::{units_of, within};

/// A structured answer handed to JavaScript as the object it is.
fn answer(value: &Value) -> JsValue {
  js_sys::JSON::parse(&value.to_string()).unwrap_or(JsValue::NULL)
}

/// A structured argument read from the JSON the package's module wrote of it.
fn argument(json: &str) -> Result<Value, JsError> {
  serde_json::from_str(json).map_err(|error| JsError::new(&format!("An argument is not JSON: {error}.")))
}

/// A field of a structured argument read as a string.
fn string(value: &Value, key: &str) -> Result<String, JsError> {
  value[key]
    .as_str()
    .map(str::to_owned)
    .ok_or_else(|| JsError::new(&format!("The field {key} of an argument is not a string.")))
}

/// A list of a structured argument, each member read by `read`.
fn list<T>(value: &Value, read: impl Fn(&Value) -> Result<T, JsError>) -> Result<Vec<T>, JsError> {
  value
    .as_array()
    .ok_or_else(|| JsError::new("An argument that is a list is not one."))?
    .iter()
    .map(read)
    .collect()
}

/// A count or an index the package passes as a number, read whole: a fraction is cut off, a negative number or
/// one that is not a number is nought, and an infinite one is the largest there is.
fn count(number: f64) -> usize {
  number as usize
}

/// A number the package answers with.
fn number(count: usize) -> f64 {
  count as f64
}

/// An index in UTF-16 units into a run of characters, as the index of the character it stands before.
fn character_at(characters: &[char], units: f64) -> usize {
  within(characters, count(units))
}

/// An index into a run of characters, in UTF-16 units.
fn units_at(characters: &[char], at: usize) -> usize {
  units_of(&characters[..at])
}

/// The index of the first member of `list` equal to `found`, so the package can give back the caller's own
/// object in place of a copy of it.
fn index_of<T: PartialEq>(list: &[T], found: &T) -> Value {
  list.iter().position(|one| one == found).map_or(Value::Null, Value::from)
}
