use serde_json::{Map, Value, json};
use wasm_bindgen::prelude::*;

use super::{answer, argument, index_of, list, string};
use crate::wording::{
  AllowedUse, BannedTerm, Glossary, KNOWN_LANGUAGES, WORDING_DOCUMENT, add_banned_terms, banned_terms_in,
  banned_terms_line, check_wording, find_banned, line_language, parse_wording,
};

// A term in the package's shape: its occasion and its own senses written only where it has them.
fn term_json(term: &BannedTerm) -> Value {
  let mut written = Map::new();

  written.insert("lang".into(), Value::from(term.lang.as_str()));
  written.insert("term".into(), Value::from(term.term.as_str()));
  written.insert("instead".into(), Value::from(term.instead.as_str()));

  if let Some(when) = &term.when {
    written.insert("when".into(), Value::from(when.as_str()));
  }

  if !term.keeps.is_empty() {
    written.insert("keeps".into(), Value::from(term.keeps.clone()));
  }

  Value::Object(written)
}

fn term_of(value: &Value) -> Result<BannedTerm, JsError> {
  Ok(BannedTerm {
    lang: string(value, "lang")?,
    term: string(value, "term")?,
    instead: string(value, "instead")?,
    when: value["when"].as_str().map(str::to_owned),
    keeps: value["keeps"]
      .as_array()
      .map_or(vec![], |keeps| keeps.iter().filter_map(Value::as_str).map(str::to_owned).collect()),
  })
}

fn allowed_json(one: &AllowedUse) -> Value {
  json!({ "lang": one.lang, "term": one.term, "path": one.path, "excerpt": one.excerpt })
}

fn allowed_of(value: &Value) -> Result<AllowedUse, JsError> {
  Ok(AllowedUse {
    lang: string(value, "lang")?,
    term: string(value, "term")?,
    path: string(value, "path")?,
    excerpt: string(value, "excerpt")?,
  })
}

fn glossary_of(value: &Value) -> Result<Glossary, JsError> {
  Ok(Glossary {
    banned: list(&value["banned"], term_of)?,
    allowed: list(&value["allowed"], allowed_of)?,
  })
}

#[wasm_bindgen(js_name = wordingDocument)]
pub fn wording_document() -> String {
  WORDING_DOCUMENT.to_owned()
}

#[wasm_bindgen(js_name = knownLanguages)]
pub fn known_languages() -> Vec<String> {
  KNOWN_LANGUAGES.iter().map(|lang| (*lang).to_owned()).collect()
}

#[wasm_bindgen(js_name = lineLanguage)]
pub fn line_language_of(line: &str) -> String {
  line_language(line).to_owned()
}

#[wasm_bindgen(js_name = parseGlossary)]
pub fn parse_glossary_of(text: &str) -> Result<JsValue, JsError> {
  let glossary = parse_wording(text).map_err(|error| JsError::new(&error))?;
  let banned: Vec<Value> = glossary.banned.iter().map(term_json).collect();
  let allowed: Vec<Value> = glossary.allowed.iter().map(allowed_json).collect();

  Ok(answer(&json!({ "banned": banned, "allowed": allowed })))
}

/// The document with the entries added, and the index among `entries` of each entry written.
#[wasm_bindgen(js_name = addBannedTerms)]
pub fn add_banned_terms_of(text: &str, entries: &str) -> Result<JsValue, JsError> {
  let entries = list(&argument(entries)?, term_of)?;
  let (text, added) = add_banned_terms(text, &entries).map_err(|error| JsError::new(&error))?;
  let added: Vec<Value> = added.iter().map(|term| index_of(&entries, term)).collect();

  Ok(answer(&json!({ "text": text, "added": added })))
}

/// Every hit in the text, its term the index of the term among `banned`.
#[wasm_bindgen(js_name = findBanned)]
pub fn find_banned_of(text: &str, banned: &str) -> Result<JsValue, JsError> {
  let banned = list(&argument(banned)?, term_of)?;
  let hits: Vec<Value> = find_banned(text, &banned)
    .iter()
    .map(|hit| json!({ "line": hit.line, "text": hit.text, "term": index_of(&banned, &hit.term) }))
    .collect();

  Ok(answer(&Value::from(hits)))
}

/// The index among `banned` of each term the text uses.
#[wasm_bindgen(js_name = bannedTermsIn)]
pub fn banned_terms_in_of(text: &str, banned: &str) -> Result<JsValue, JsError> {
  let banned = list(&argument(banned)?, term_of)?;
  let used: Vec<Value> =
    banned_terms_in(text, &banned).iter().map(|term| index_of(&banned, term)).collect();

  Ok(answer(&Value::from(used)))
}

#[wasm_bindgen(js_name = bannedTermsLine)]
pub fn banned_terms_line_of(terms: &str) -> Result<String, JsError> {
  Ok(banned_terms_line(&list(&argument(terms)?, term_of)?))
}

/// The report of the files, each read through `read`: each hit with its term's index among the glossary's
/// banned terms, and the index among its allowed uses of each one left unused. An error `read` throws is
/// thrown on.
#[wasm_bindgen(js_name = checkWording)]
pub fn check_wording_of(
  paths: Vec<String>,
  read: &js_sys::Function,
  glossary: &str,
) -> Result<JsValue, JsValue> {
  let glossary = glossary_of(&argument(glossary)?)?;
  let paths: Vec<&str> = paths.iter().map(String::as_str).collect();
  let read = |path: &str| {
    read
      .call1(&JsValue::NULL, &JsValue::from(path))
      .map(|text| text.as_string().unwrap_or_default())
  };
  let report = check_wording(&paths, read, &glossary)?;
  let hits: Vec<Value> = report
    .hits
    .iter()
    .map(|found| {
      json!({
        "path": found.path,
        "line": found.hit.line,
        "text": found.hit.text,
        "term": index_of(&glossary.banned, &found.hit.term),
      })
    })
    .collect();
  let unused: Vec<Value> =
    report.unused_allowances.iter().map(|one| index_of(&glossary.allowed, one)).collect();

  Ok(answer(&json!({ "hits": hits, "unusedAllowances": unused })))
}
