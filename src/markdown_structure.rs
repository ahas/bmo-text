//! The structure of a markdown text, read off its source, the twin of the package's `markdown-structure`: its
//! headings, and where it may be cut. A table of contents, the marks stamped on drawn headings, the parts a
//! translator is given and the count that tells a translation that kept its structure all read the headings
//! here, so a heading is one thing to all of them.
//!
//! Only ATX headings count, a line of one to six `#`. A line inside a fenced code block is code, whatever it
//! starts with, and no text is cut inside one. The regular expressions are the package's, with JavaScript's
//! classes spelled out, and those that look around or refer back are run by a backtracking engine as
//! JavaScript runs them.

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use regex::Regex;
use unicode_normalization::UnicodeNormalization;

use crate::js::{self, SPACE, units};

/// One ATX heading of a text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkdownHeading {
  /// 1 for `#`, 6 for `######`.
  pub depth: usize,
  /// What the heading says, with its inline markup taken off.
  pub text: String,
  /// A mark no other heading of the same text carries, made from its words: the first of a text takes the
  /// words alone, the ones after it a number as well.
  pub slug: String,
  /// The source line the heading stands on, counted from 1.
  pub line: usize,
}

/// A text cut in two, and what joins the halves back: a blank line where the cut fell at a paragraph, a line
/// break where it fell at a line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Halved {
  pub halves: (String, String),
  pub joint: &'static str,
}

static ATX: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(&format!(r"^ {{0,3}}(#{{1,6}})(?:[ \t]+({}*?))?(?:[ \t]+#+)?[ \t]*$", js::DOT))
    .expect("the heading line")
});
static FENCE: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^ {0,3}(`{3,}|~{3,})").expect("the fence line"));
static LINK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"!?\[([^\]]*)\]\([^)]*\)").expect("a link"));
static CODE_MARKS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"`+").expect("code marks"));
static PAIRED: LazyLock<fancy_regex::Regex> = LazyLock::new(|| {
  fancy_regex::Regex::new(&format!(r"(\*\*|__|~~)({}+?)\1", js::DOT)).expect("a paired mark")
});
static EMPHASIS: LazyLock<fancy_regex::Regex> = LazyLock::new(|| {
  fancy_regex::Regex::new(&format!(
    r"(^|[^A-Za-z0-9_])[*_]([^{SPACE}](?:{}*?[^{SPACE}])?)[*_](?=[^A-Za-z0-9_]|$)",
    js::DOT
  ))
  .expect("an emphasis")
});
static ESCAPED: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"\\([\\`*_{}\[\]()#+\-.!~])").expect("an escaped character"));
static UNSLUGGED: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(&format!(r"[^\p{{L}}\p{{N}}{SPACE}_-]")).expect("what a mark drops"));
static SPACES: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(&format!("[{SPACE}]+")).expect("a run of space"));
static LINE_ENDS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\r\n?").expect("a line end"));

/// A heading's words without the markup around them: a link or an image stands as its words, and emphasis,
/// strikes and code marks fall away.
pub fn plain_heading(text: &str) -> String {
  let text = LINK.replace_all(text, "${1}");
  let text = CODE_MARKS.replace_all(&text, "");
  let text = PAIRED.replace_all(&text, "${2}");
  let text = EMPHASIS.replace_all(&text, "${1}${2}");
  let text = ESCAPED.replace_all(&text, "${1}");

  js::trim(&text).to_owned()
}

/// The words of a heading as a mark: lower-case, letters and digits of any script kept, runs of space made one
/// hyphen, everything else dropped; `section` where nothing is left.
pub fn heading_slug(text: &str) -> String {
  let lowered = text.nfc().collect::<String>().to_lowercase();
  let kept = UNSLUGGED.replace_all(&lowered, "");
  let slug = SPACES.replace_all(js::trim(&kept), "-");

  if slug.is_empty() { "section".to_owned() } else { slug.into_owned() }
}

// The fence a line opens or closes a code block with, as far as a line alone can say.
fn fence_of(line: &str) -> Option<&str> {
  FENCE.captures(line).and_then(|found| found.get(1)).map(|fence| fence.as_str())
}

// Whether `marker`, standing on `line`, closes the block `fence` opened.
fn closes(fence: &str, marker: &str, line: &str) -> bool {
  marker.as_bytes()[0] == fence.as_bytes()[0] && marker.len() >= fence.len() && js::trim(line) == marker
}

/// Every heading of a text, in the order it stands. A heading with no words is no entry.
pub fn markdown_headings(source: &str) -> Vec<MarkdownHeading> {
  let mut headings = vec![];
  let mut taken = HashSet::new();
  let mut counts: HashMap<String, usize> = HashMap::new();
  let mut fence: Option<String> = None;
  let source = LINE_ENDS.replace_all(source, "\n");

  for (index, line) in source.split('\n').enumerate() {
    let opening = fence_of(line);

    if let Some(open) = &fence {
      if opening.is_some_and(|marker| closes(open, marker, line)) {
        fence = None;
      }

      continue;
    }

    if let Some(marker) = opening {
      fence = Some(marker.to_owned());
      continue;
    }

    let Some(atx) = ATX.captures(line) else {
      continue;
    };
    let text = plain_heading(atx.get(2).map_or("", |words| words.as_str()));

    if text.is_empty() {
      continue;
    }

    let base = heading_slug(&text);
    let mut slug = base.clone();
    let mut count = counts.get(&base).copied().unwrap_or(0);

    while taken.contains(&slug) {
      count += 1;
      slug = format!("{base}-{count}");
    }

    counts.insert(base, count);
    taken.insert(slug.clone());
    headings.push(MarkdownHeading {
      depth: atx[1].len(),
      text,
      slug,
      line: index + 1,
    });
  }

  headings
}

// A place a text may be cut, where the second half starts: its UTF-16 offset and its byte offset.
#[derive(Clone, Copy)]
struct Cut {
  units: usize,
  byte: usize,
}

// Where a text may be cut in two: after a blank line outside any code block, or failing that at a line break
// outside one.
fn cut_points(text: &str) -> (Vec<Cut>, Vec<Cut>) {
  let mut paragraphs = vec![];
  let mut lines = vec![];
  let mut fence: Option<String> = None;
  let mut cut = Cut { units: 0, byte: 0 };
  let all: Vec<&str> = text.split('\n').collect();
  let length = units(text);

  for (index, line) in all.iter().enumerate() {
    let marker = fence_of(line);

    match (&fence, marker) {
      (Some(open), Some(marker)) if closes(open, marker, line) => fence = None,
      (None, Some(marker)) => fence = Some(marker.to_owned()),
      _ => {}
    }

    cut.units += units(line) + 1;
    cut.byte += line.len() + 1;

    if fence.is_none() && index < all.len() - 1 && cut.units < length {
      lines.push(cut);

      if js::trim(line).is_empty() {
        paragraphs.push(cut);
      }
    }
  }

  (paragraphs, lines)
}

/// A text in two, at the paragraph boundary nearest its middle, or the line boundary nearest it where there is
/// no paragraph boundary, with what joins the halves back; none where it is one line. The middle and the
/// distances are in UTF-16 units, the earlier of two boundaries as near winning.
pub fn halve_text(text: &str) -> Option<Halved> {
  let (paragraphs, lines) = cut_points(text);
  let points = if paragraphs.is_empty() { &lines } else { &paragraphs };
  let length = units(text);
  // Twice the distance from the middle, so the middle of an odd length needs no fraction.
  let distance = |cut: &Cut| (2 * cut.units).abs_diff(length);
  let cut = points
    .iter()
    .copied()
    .reduce(|best, point| if distance(&point) < distance(&best) { point } else { best })?;

  Some(Halved {
    halves: (text[..cut.byte].to_owned(), text[cut.byte..].to_owned()),
    joint: if paragraphs.is_empty() { "\n" } else { "\n\n" },
  })
}

/// A text as its sections: the lines before the first heading, then each heading with the lines under it up to
/// the next heading of any depth. Joined back, the sections are the text.
pub fn markdown_sections(source: &str) -> Vec<String> {
  let lines: Vec<&str> = source.split('\n').collect();
  // As JavaScript's `slice`, a range past the end is cut to the end.
  let slice = |from: usize, to: usize| {
    lines[from.min(lines.len())..to.clamp(from.min(lines.len()), lines.len())].join("\n")
  };
  let mut sections = vec![];
  let mut from = 0;

  for start in markdown_headings(source)
    .iter()
    .map(|heading| heading.line - 1)
    .filter(|start| *start > 0)
  {
    sections.push(format!("{}\n", slice(from, start)));
    from = start;
  }

  sections.push(slice(from, lines.len()));
  sections
}
