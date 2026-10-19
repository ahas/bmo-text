//! The web addresses a text names, found the way a chat app finds them. A markdown link names the address its
//! parentheses hold and nothing after them. A bare http or https address runs to the next space or Korean
//! letter, and what the sentence put after it — a full stop, a comma, a closing bracket that opened before
//! the address — is the sentence's. A pair of brackets inside the address, as a wiki page's name carries,
//! stays in it. Positions are character indices into the text as `js::chars` reads it.

use std::sync::LazyLock;

use regex::Regex;

use crate::js::{SPACE, chars, is_space, text};

// Korean letters: jamo, compatibility jamo, and the syllables with their extensions.
const HANGUL: &str =
  r"\x{1100}-\x{11ff}\x{3130}-\x{318f}\x{a960}-\x{a97f}\x{ac00}-\x{d7af}\x{d7b0}-\x{d7ff}";

static BARE: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(&format!(r#"(?i)^https?://[^{SPACE}<>"'`{HANGUL}]+"#)).expect("a bare address")
});
static SENTENCE_LETTER: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(&format!(r#"[{SPACE}<>"'`{HANGUL}]"#)).expect("a sentence's letter"));
static GLUED: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(&format!(r"^[)\]}}.,;:!?]*[{HANGUL}]")).expect("a glued letter"));
static TRAILING: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new("[.,;:!?'\"\u{2026}\u{3002}\u{3001}\u{300d}\u{300f}\u{300b}\u{ff09}]+$")
    .expect("trailing marks")
});

const BRACKETS: [(char, char); 3] = [('(', ')'), ('[', ']'), ('{', '}')];

/// A candidate address as it ends, with what the sentence put after it taken off: everything from the first
/// space or Korean letter on, the punctuation at its end, and each closing bracket it did not open.
pub fn address_as_written(candidate: &str) -> String {
  let cut = SENTENCE_LETTER.find(candidate).map_or(candidate, |letter| &candidate[..letter.start()]);
  let mut url = TRAILING.replace(cut, "").into_owned();

  for (open, close) in BRACKETS {
    while url.ends_with(close) && url.matches(open).count() < url.matches(close).count() {
      url.pop();
      url = TRAILING.replace(&url, "").into_owned();
    }
  }

  url
}

/// Whether a text reads as an http or https address with a host, as a browser reads one.
pub fn is_web_address(url: &str) -> bool {
  url::Url::parse(url).is_ok_and(|parsed| {
    matches!(parsed.scheme(), "http" | "https") && parsed.host_str().is_some_and(|host| !host.is_empty())
  })
}

/// A markdown link: its label, its address, and the index just past its closing parenthesis.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkdownLink {
  pub label: String,
  pub url: String,
  pub end: usize,
}

/// `[label](url)` starting at `at`, where `at` is the opening bracket. Brackets in the label and one level
/// of parentheses in the address are balanced; the address holds no space.
pub fn markdown_link_at(text: &[char], at: usize) -> Option<MarkdownLink> {
  let mut depth = 0i64;
  let mut close = None;
  let mut index = at;

  while index < text.len() {
    match text[index] {
      '\\' => index += 1,
      '[' => depth += 1,
      ']' => {
        depth -= 1;

        if depth == 0 {
          close = Some(index);
          break;
        }
      }
      _ => {}
    }

    index += 1;
  }

  let close = close?;

  if text.get(close + 1) != Some(&'(') {
    return None;
  }

  let mut parens = 0;

  for index in close + 2..text.len() {
    match text[index] {
      character if is_space(character) => return None,
      '(' => parens += 1,
      ')' if parens == 0 => {
        let url = self::text(&text[close + 2..index]);

        return (!url.is_empty()).then(|| MarkdownLink {
          label: self::text(&text[at + 1..close]),
          url,
          end: index + 1,
        });
      }
      ')' => parens -= 1,
      _ => {}
    }
  }

  None
}

/// The bare address that starts exactly at `at`, cut where the sentence takes over, or none.
pub fn bare_address_at(text: &[char], at: usize) -> Option<String> {
  let rest = self::text(&text[at.min(text.len())..]);
  let found = BARE.find(&rest)?;
  let url = address_as_written(found.as_str());

  (!url.is_empty() && is_web_address(&url)).then_some(url)
}

/// Whether the sentence runs on with no space after the address that ends at `end`: a Korean letter straight
/// after it, or after the closing brackets and punctuation it left behind.
pub fn glued_after_address(text: &[char], end: usize) -> bool {
  GLUED.is_match(&self::text(&text[end.min(text.len())..]))
}

/// Where a web address stands in a text, end exclusive.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FoundAddress {
  pub start: usize,
  pub end: usize,
  pub url: String,
}

/// Every web address a text names, in the order written. A markdown link gives the address in its
/// parentheses and nothing else.
pub fn web_addresses(text: &[char]) -> Vec<FoundAddress> {
  let mut found = vec![];
  let mut index = 0;

  while index < text.len() {
    let character = text[index];

    if character == '['
      && let Some(link) = markdown_link_at(text, index)
    {
      let length = link.url.chars().count();

      if is_web_address(&link.url) {
        found.push(FoundAddress {
          start: link.end - 1 - length,
          end: link.end - 1,
          url: link.url,
        });
      }

      index = link.end;
      continue;
    }

    if (character == 'h' || character == 'H')
      && let Some(url) = bare_address_at(text, index)
    {
      let length = url.chars().count();

      found.push(FoundAddress {
        start: index,
        end: index + length,
        url,
      });
      index += length;
      continue;
    }

    index += 1;
  }

  found
}

/// The addresses a text names, read off its characters.
pub fn web_addresses_in(text: &str) -> Vec<FoundAddress> {
  web_addresses(&chars(text))
}
