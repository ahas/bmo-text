//! The terms a glossary bans (`WORDING.md` by default), read out of the document and found in a text: the
//! net every writer of the system's own words is held to, outside the spans a text quotes.

use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

use crate::js::units;

/// A span a written text names a wrong form or quotes a command or somebody's words in: inside backticks or a
/// pair of quotation marks, on one line.
pub static QUOTED_SPAN: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r#"`[^`\n]*`|"[^"\n]*"|'[^'\n]*'|“[^”\n]*”|‘[^’\n]*’|「[^」\n]*」|『[^』\n]*』"#)
    .expect("a quoted span")
});

/// A term the glossary bans, with what to write instead, when the entry applies where its
/// line says so, and the phrases in which it keeps its own sense and passes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BannedTerm {
  pub lang: String,
  pub term: String,
  pub instead: String,
  pub when: Option<String>,
  pub keeps: Vec<String>,
}

/// A use of a banned term the document allows in one file, on the line holding the excerpt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AllowedUse {
  pub lang: String,
  pub term: String,
  pub path: String,
  pub excerpt: String,
}

/// The document's lists: the banned terms and the uses it allows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Glossary {
  pub banned: Vec<BannedTerm>,
  pub allowed: Vec<AllowedUse>,
}

/// One line of a text using a banned term: its number, counted from one, the line and the term.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WordingHit {
  pub line: usize,
  pub text: String,
  pub term: BannedTerm,
}

/// The document the banned words are read out of, at the source root.
pub const WORDING_DOCUMENT: &str = "WORDING.md";

static BANNED_LINE: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^- ([a-z]{2,3}) · (.+?) → (.+?)(?: — (.*))?$").unwrap());
static ALLOWED_LINE: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^- ([a-z]{2,3}) · (.+?) · (\S+) · (.+) — .+$").unwrap());
static OWN_SENSE_LINE: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^- ([a-z]{2,3}) · (.+?) · (.+?) — .+$").unwrap());

// The languages a line can be told to be in, by the script it is written in.
const KNOWN_LANGUAGES: [&str; 3] = ["ko", "ja", "en"];

/// The language a line is written in, told by its script: Korean where it holds Hangul, Japanese where it
/// holds kana alone, English otherwise.
pub fn line_language(line: &str) -> &'static str {
  let korean = |c: char| matches!(c, 'ᄀ'..='ᇿ' | '㄰'..='㆏' | '가'..='힯');
  let japanese = |c: char| matches!(c, '぀'..='ヿ');

  if line.chars().any(korean) {
    "ko"
  } else if line.chars().any(japanese) {
    "ja"
  } else {
    "en"
  }
}

/// The banned terms out of the document's own sections. A line naming a language no line can be told
/// in, or a phrase of a word's own sense that names no banned term or does not hold it, is a fault in
/// the document.
pub fn parse_glossary(text: &str) -> Result<Vec<BannedTerm>, String> {
  parse_wording(text).map(|glossary| glossary.banned)
}

/// The document's lists out of its own sections, held to the rules `parse_glossary` holds them to.
pub fn parse_wording(text: &str) -> Result<Glossary, String> {
  let mut banned: Vec<BannedTerm> = Vec::new();
  let mut allowed: Vec<AllowedUse> = Vec::new();
  let mut section = "";
  let checked = |lang: &str, line: &str| -> Result<String, String> {
    if KNOWN_LANGUAGES.contains(&lang) {
      Ok(lang.to_owned())
    } else {
      Err(format!("{WORDING_DOCUMENT}: unknown language code \"{lang}\" in: {line}"))
    }
  };

  for line in text.split('\n') {
    if let Some(title) = line.strip_prefix("## ") {
      section = title.trim();
      continue;
    }

    if section == "Banned terms" {
      if let Some(found) = BANNED_LINE.captures(line) {
        banned.push(BannedTerm {
          lang: checked(&found[1], line)?,
          term: found[2].to_owned(),
          instead: found[3].to_owned(),
          when: found.get(4).map(|when| when.as_str().to_owned()),
          keeps: Vec::new(),
        });
      }
    } else if section == "Allowed uses" {
      if let Some(found) = ALLOWED_LINE.captures(line) {
        allowed.push(AllowedUse {
          lang: checked(&found[1], line)?,
          term: found[2].to_owned(),
          path: found[3].to_owned(),
          excerpt: found[4].to_owned(),
        });
      }
    } else if section == "Uses in the word's own sense"
      && let Some(found) = OWN_SENSE_LINE.captures(line)
    {
      let lang = checked(&found[1], line)?;
      let (term, phrase) = (&found[2], &found[3]);
      let entry = banned.iter_mut().find(|one| one.lang == lang && one.term == term);

      match entry {
        Some(entry) if phrase.contains(term) => entry.keeps.push(phrase.to_owned()),
        _ => {
          return Err(format!(
            "{WORDING_DOCUMENT}: a phrase of a word's own sense must hold a banned term listed above it: {line}"
          ));
        }
      }
    }
  }

  Ok(Glossary { banned, allowed })
}

/// The banned terms as the document at the source root lists them, or none where it cannot be read: a
/// report is not held to a document nobody can read.
pub fn read_glossary(source: &Path) -> Vec<BannedTerm> {
  std::fs::read_to_string(source.join(WORDING_DOCUMENT))
    .ok()
    .and_then(|text| parse_glossary(&text).ok())
    .unwrap_or_default()
}

fn hangul_syllable(c: char) -> bool {
  matches!(c, '가'..='힣')
}

// Whether the term stands on its own where it was found, rather than being the tail of a longer word:
// a Korean term another syllable runs into from the left belongs to that other word.
fn stands_alone(line: &str, at: usize, term: &BannedTerm) -> bool {
  if at == 0 || !term.term.chars().next().is_some_and(hangul_syllable) {
    return true;
  }

  !line[..at].chars().next_back().is_some_and(hangul_syllable)
}

// Whether the term, where it was found, is part of a phrase in which it keeps its own sense.
fn keeps_own_sense(line: &str, at: usize, term: &BannedTerm) -> bool {
  term.keeps.iter().any(|phrase| {
    phrase
      .find(&term.term)
      .and_then(|inside| at.checked_sub(inside))
      .is_some_and(|start| line.get(start..).is_some_and(|rest| rest.starts_with(phrase.as_str())))
  })
}

// Whether a line of the term's own language uses it anywhere it neither runs into a longer word nor
// keeps its own sense.
fn used_in(line: &str, term: &BannedTerm) -> bool {
  let mut from = 0;

  while let Some(found) = line[from..].find(&term.term) {
    let at = from + found;

    if stands_alone(line, at, term) && !keeps_own_sense(line, at, term) {
      return true;
    }
    from = at + line[at..].chars().next().map_or(1, char::len_utf8);
  }

  false
}

/// The banned terms a text uses outside its quoted spans, each once and in the glossary's order.
pub fn banned_terms_in(text: &str, glossary: &[BannedTerm]) -> Vec<BannedTerm> {
  let bare = QUOTED_SPAN.replace_all(text, |found: &regex::Captures| " ".repeat(units(&found[0])));
  let lines: Vec<(&str, &str)> = bare.split('\n').map(|line| (line_language(line), line)).collect();

  glossary
    .iter()
    .filter(|term| lines.iter().any(|(lang, line)| *lang == term.lang && used_in(line, term)))
    .cloned()
    .collect()
}

/// The terms in one line, each beside what to write: `"<term>" → <what to write>; …`.
pub fn banned_terms_line(terms: &[BannedTerm]) -> String {
  terms
    .iter()
    .map(|term| format!("\"{}\" → {}", term.term, term.instead))
    .collect::<Vec<_>>()
    .join("; ")
}

/// Every use of a banned term in a text, a line at a time and on the lines of the term's own language,
/// quoted spans and all: what a file of the repository is held to.
pub fn find_banned(text: &str, banned: &[BannedTerm]) -> Vec<WordingHit> {
  let mut hits = Vec::new();

  for (index, line) in text.split('\n').enumerate() {
    let lang = line_language(line);

    for term in banned.iter().filter(|term| term.lang == lang && used_in(line, term)) {
      hits.push(WordingHit {
        line: index + 1,
        text: line.to_owned(),
        term: term.clone(),
      });
    }
  }

  hits
}

// A banned term's line in the list's own shape.
fn banned_line(term: &BannedTerm) -> String {
  let when = term.when.as_ref().map(|when| format!(" — {when}")).unwrap_or_default();

  format!("- {} · {} → {}{when}", term.lang, term.term, term.instead)
}

/// The document with new banned lines written after the last entry of its list, and the entries written.
/// A term the list already bans in that language is passed over rather than written twice, and a line that
/// would not read back as the entry it was meant to be is refused, so nothing is written that the check
/// would then read otherwise.
pub fn add_banned_terms(text: &str, entries: &[BannedTerm]) -> Result<(String, Vec<BannedTerm>), String> {
  let held = parse_glossary(text)?;
  let mut added: Vec<BannedTerm> = Vec::new();

  for entry in entries {
    let known = |one: &BannedTerm| one.lang == entry.lang && one.term == entry.term;

    if !held.iter().any(known) && !added.iter().any(known) {
      added.push(entry.clone());
    }
  }

  if added.is_empty() {
    return Ok((text.to_owned(), added));
  }

  let lines: Vec<&str> = text.split('\n').collect();
  let heading = lines
    .iter()
    .position(|line| *line == "## Banned terms")
    .ok_or_else(|| format!("{WORDING_DOCUMENT}: there is no \"## Banned terms\" list to add to"))?;
  let mut end = lines
    .iter()
    .skip(heading + 1)
    .position(|line| line.starts_with("## "))
    .map_or(lines.len(), |at| heading + 1 + at);

  while end > heading + 1 && lines[end - 1].trim().is_empty() {
    end -= 1;
  }

  let written: Vec<String> = added.iter().map(banned_line).collect();
  let result = lines[..end]
    .iter()
    .map(|line| (*line).to_owned())
    .chain(written)
    .chain(lines[end..].iter().map(|line| (*line).to_owned()))
    .collect::<Vec<_>>()
    .join("\n");
  let reread = parse_glossary(&result)?;

  for entry in &added {
    let back = reread.iter().find(|one| one.lang == entry.lang && one.term == entry.term);

    if back.is_none_or(|back| back.instead != entry.instead || back.when != entry.when) {
      return Err(format!(
        "{WORDING_DOCUMENT}: the line for \"{}\" would not read back as written",
        entry.term
      ));
    }
  }

  Ok((result, added))
}
