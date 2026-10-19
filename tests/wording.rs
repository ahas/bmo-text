mod shared;

use std::path::Path;

use bmo_markup::wording::*;
use serde_json::{Map, Value, json};

use shared::text;

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

fn terms_json(terms: &[BannedTerm]) -> Value {
  terms.iter().map(term_json).collect()
}

fn term_of(value: &Value) -> BannedTerm {
  BannedTerm {
    lang: text(&value["lang"]).to_owned(),
    term: text(&value["term"]).to_owned(),
    instead: text(&value["instead"]).to_owned(),
    when: value["when"].as_str().map(str::to_owned),
    keeps: value["keeps"]
      .as_array()
      .map_or(vec![], |keeps| keeps.iter().map(|one| text(one).to_owned()).collect()),
  }
}

fn terms_of(value: &Value) -> Vec<BannedTerm> {
  value.as_array().expect("a list of terms").iter().map(term_of).collect()
}

fn refused(error: String) -> Value {
  json!({ "error": error })
}

#[test]
fn the_shared_cases_hold() {
  shared::hold_all(
    "wording",
    &[
      ("lineLanguage", &|args| Value::from(line_language(text(&args[0])))),
      ("parseGlossary", &|args| {
        parse_wording(text(&args[0])).map_or_else(refused, |glossary| {
          let allowed: Vec<Value> = glossary
            .allowed
            .iter()
            .map(
              |one| json!({ "lang": one.lang, "term": one.term, "path": one.path, "excerpt": one.excerpt }),
            )
            .collect();

          json!({ "banned": terms_json(&glossary.banned), "allowed": allowed })
        })
      }),
      ("findBanned", &|args| {
        // The package's hit carries the path the text was read from, which the caller passes it.
        let hit = |hit: &WordingHit| json!({ "path": args[0], "line": hit.line, "text": hit.text, "term": term_json(&hit.term) });

        find_banned(text(&args[1]), &terms_of(&args[2])).iter().map(hit).collect()
      }),
      ("bannedTermsIn", &|args| {
        terms_json(&banned_terms_in(text(&args[0]), &terms_of(&args[1])))
      }),
      ("bannedTermsLine", &|args| Value::from(banned_terms_line(&terms_of(&args[0])))),
      ("addBannedTerms", &|args| {
        add_banned_terms(text(&args[0]), &terms_of(&args[1]))
          .map_or_else(refused, |(text, added)| json!({ "text": text, "added": terms_json(&added) }))
      }),
    ],
  );
}

#[test]
fn a_folder_with_no_glossary_bans_nothing() {
  assert!(read_glossary(Path::new("/nonexistent/folder")).is_empty());
}
