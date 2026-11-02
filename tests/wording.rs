use crate::shared;

use std::path::Path;

use bmo_text::wording::*;
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
      ("swapsIn", &|args| {
        let hit = |hit: &WordingHit| json!({ "path": args[0], "line": hit.line, "text": hit.text, "term": term_json(&hit.term) });

        swaps_in(text(&args[1]), &terms_of(&args[2])).iter().map(hit).collect()
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

const SAMPLE: &str = "## Banned terms\n\n- en · tool strip → toolbar\n- en · fixing → editing — when it means editing\n\n## Allowed uses\n\n- en · fixing · docs/a.md · bike fixing — a repair";

#[test]
fn an_allowed_use_passes_only_in_its_own_file_and_line_and_a_stale_one_is_reported() {
  let glossary = parse_wording(SAMPLE).expect("the sample reads");
  let files = [
    ("docs/a.md", "bike fixing\ndocument fixing\n"),
    ("docs/b.md", "bike fixing\nclick the tool strip\n"),
  ];
  let read = |path: &str| {
    files
      .iter()
      .find(|(name, _)| *name == path)
      .map(|(_, text)| (*text).to_owned())
      .ok_or(())
  };
  let report = check_wording(&["docs/a.md", "docs/b.md"], read, &glossary).expect("every file reads");
  let hits: Vec<String> = report
    .hits
    .iter()
    .map(|found| format!("{}:{}:{}", found.path, found.hit.line, found.hit.term.term))
    .collect();

  assert_eq!(hits, ["docs/a.md:2:fixing", "docs/b.md:1:fixing", "docs/b.md:2:tool strip"]);
  assert!(report.unused_allowances.is_empty());
  assert_eq!(check_wording(&[], read, &glossary).expect("nothing to read").unused_allowances.len(), 1);
  assert_eq!(check_wording(&["docs/c.md"], read, &glossary), Err(()));
}

#[test]
fn a_folder_with_no_glossary_bans_nothing() {
  assert!(read_glossary(Path::new("/nonexistent/folder")).is_empty());
}

// A folder of its own for one test, holding the glossary document given, taken down when the test ends.
struct GlossaryFolder(std::path::PathBuf);

impl GlossaryFolder {
  fn holding(name: &str, document: &str) -> Self {
    let folder = std::env::temp_dir().join(format!("bmo-text-wording-{name}-{}", std::process::id()));

    std::fs::create_dir_all(&folder).expect("the folder is made");
    std::fs::write(folder.join(WORDING_DOCUMENT), document).expect("the glossary is written");
    Self(folder)
  }
}

impl Drop for GlossaryFolder {
  fn drop(&mut self) {
    let _ = std::fs::remove_dir_all(&self.0);
  }
}

#[test]
fn a_text_is_held_to_the_glossary_in_the_folder_named() {
  let folder = GlossaryFolder::holding("read", SAMPLE);
  let hits =
    text_swaps("the `tool strip` is gone\nclick the tool strip", &folder.0).expect("the glossary reads");

  assert_eq!(hits.len(), 1);
  assert_eq!((hits[0].line, hits[0].term.instead.as_str()), (2, "toolbar"));
}

#[test]
fn a_glossary_that_cannot_be_read_or_parsed_is_said_rather_than_read_as_clean() {
  let missing = text_swaps("anything", Path::new("/nonexistent/folder")).expect_err("no glossary there");
  let faulty = GlossaryFolder::holding("faulty", "## Banned terms\n\n- xx · tool strip → toolbar\n");

  assert!(
    missing.starts_with("The glossary /nonexistent/folder/WORDING.md cannot be read"),
    "{missing}"
  );
  assert!(
    text_swaps("anything", &faulty.0)
      .expect_err("a faulty glossary")
      .contains("unknown language code")
  );
}
