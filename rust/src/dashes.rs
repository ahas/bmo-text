//! A dash left between words, replaced the way a person types the same sentence (prompts/rules/
//! plain-writing.md). The writer is told never to write one; this is the net under that rule on the way out,
//! so it reads nothing but the characters: a name and its explanation get a colon, an aside between two
//! dashes of one sentence gets commas (or brackets, where it has commas of its own), every further dash in
//! the sentence a comma, and a dash between two digits is a range. What is someone else's or is not prose
//! keeps its dashes: a code block or span, a quoted line, a web address, a dash standing alone in a table
//! cell or at either end of a paragraph.
//!
//! A chat answer's line break is a line of its own on the screen, so a sentence ends with its line. A
//! document wrapped at a fixed width is read with each paragraph or list item as one piece.

use std::sync::LazyLock;

use regex::Regex;

use crate::js::{self, SPACE, chars};
use crate::web_address::web_addresses;

static FENCE: LazyLock<Regex> = LazyLock::new(|| Regex::new(&format!("^[{SPACE}]*```")).expect("a fence"));
static QUOTE: LazyLock<Regex> = LazyLock::new(|| Regex::new(&format!("^[{SPACE}]*>")).expect("a quote"));
// A line that opens a block of its own rather than running on from the one above it.
static BLOCK_START: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(&format!(
    "^[{SPACE}]*(?:$|[-*+\u{2022}][{SPACE}]|[0-9]{{1,3}}[.)][{SPACE}]|[\u{2460}-\u{2473}]|#|\\||>|```)"
  ))
  .expect("a block's start")
});
// A block that is one line whatever follows it.
static ONE_LINE_BLOCK: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(&format!("^[{SPACE}]*(?:#|\\|)")).expect("a one-line block"));
// What may follow a sentence's closing mark: any footnote mark, emphasis, quote or bracket closing on it,
// then a space or the text's end.
static AFTER_SENTENCE: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(&format!(
    "^(?:\\[[^\\]{SPACE}]{{1,8}}\\])*[*_\"'\u{201d}\u{2019})\\]\u{300d}\u{300f}]*(?:[{SPACE}]|$)"
  ))
  .expect("a sentence's end")
});
// A line's opening list mark, which is not a word the dash could stand after.
static LIST_MARK: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(&format!("^[{SPACE}]*(?:[-*+\u{2022}]|[0-9]{{1,3}}[.)])$")).expect("a list mark")
});
// A label opening a line, such as "TITLE: ": its colon names the line, and the sentence after it may still
// take one. The space after the colon is matched and left out of the label.
static LABEL: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(&format!(
    "^[{SPACE}]*(?:[-*+\u{2022}][{SPACE}]+|[0-9]{{1,3}}[.)][{SPACE}]+)?[^{SPACE}:.,;!?]+(?: [^{SPACE}:.,;!?]+){{0,2}}:[{SPACE}]"
  ))
  .expect("a label")
});
static QUOTED: LazyLock<Regex> =
  LazyLock::new(|| Regex::new("\"[^\"]*\"|\u{201c}[^\u{201d}]*\u{201d}").expect("a quotation"));

// Marks after which the dash only doubled a pause the sentence already has.
fn pause(character: Option<&char>) -> bool {
  matches!(character, Some('.' | ',' | ':' | ';' | '!' | '?' | '\u{3002}' | '\u{3001}'))
}

fn opener(character: Option<&char>) -> bool {
  matches!(character, Some('(' | '[' | '{' | '\u{201c}' | '\u{2018}' | '\u{300c}' | '\u{300e}'))
}

fn closer(character: Option<&char>) -> bool {
  matches!(character, Some(')' | ']' | '}' | '\u{201d}' | '\u{2019}' | '\u{300d}' | '\u{300f}'))
}

// Marks inside an aside that commas around it would run together with.
fn aside_pause(character: &char) -> bool {
  matches!(character, ',' | ';' | ':')
}

fn is_space(character: Option<&char>) -> bool {
  matches!(character, Some(' ' | '\t' | '\n'))
}

fn is_digit(character: Option<&char>) -> bool {
  character.is_some_and(char::is_ascii_digit)
}

fn is_dash(character: char) -> bool {
  matches!(character, '\u{2014}' | '\u{2015}')
}

// An em dash or a horizontal bar, or an en dash with a space either side; an en dash touching its words is a
// range and stays.
fn any_dash(text: &str) -> bool {
  text.chars().any(is_dash) || text.contains(" \u{2013} ")
}

// Each dash of a piece: a run of em dashes or bars, or one spaced en dash.
fn dash_runs(piece: &[char]) -> Vec<(usize, usize)> {
  let mut runs = vec![];
  let mut at = 0;

  while at < piece.len() {
    if is_dash(piece[at]) {
      let end = at + piece[at..].iter().take_while(|one| is_dash(**one)).count();

      runs.push((at, end));
      at = end;
    } else if piece[at] == '\u{2013}' && at > 0 && piece[at - 1] == ' ' && piece.get(at + 1) == Some(&' ') {
      runs.push((at, at + 1));
      at += 1;
    } else {
      at += 1;
    }
  }

  runs
}

struct Dash {
  start: usize,
  end: usize,
  // The nearest character that is not white space on each side.
  left: usize,
  right: usize,
  sentence: usize,
}

struct Swap {
  from: usize,
  to: usize,
  text: String,
}

// The stretches of a piece that are not the writer's prose: code spans, web addresses, and words in double
// quotes, each quote paired within its line.
fn kept_spans(piece: &[char]) -> Vec<(usize, usize)> {
  let mut spans: Vec<(usize, usize)> =
    web_addresses(piece).iter().map(|found| (found.start, found.end)).collect();
  let mut offset = 0;

  for line in js::text(piece).split('\n') {
    for quoted in QUOTED.find_iter(line) {
      let start = offset + line[..quoted.start()].chars().count();

      spans.push((start, start + quoted.as_str().chars().count()));
    }

    offset += line.chars().count() + 1;
  }

  let mut at = 0;

  while at < piece.len() {
    if piece[at] != '`' {
      at += 1;
      continue;
    }

    let run = piece[at..].iter().take_while(|one| **one == '`').count();

    let Some(close) = js::find(piece, &vec!['`'; run], at + run) else {
      at += run;
      continue;
    };

    spans.push((at, close + run));
    at = close + run;
  }

  spans
}

// What takes a dash's place with the white space around it: a mark that holds on to the word before, a mark
// that holds on to the word after, and between them a space, or the line break and indent of the line the
// text goes on in where the dash stood at a wrap.
fn swap_at(piece: &[char], dash: &Dash, before: &str, after: &str) -> Swap {
  let ahead = &piece[dash.left + 1..dash.start];
  let behind = &piece[dash.end..dash.right];
  let from_break = |side: &[char]| side.iter().position(|one| *one == '\n').map(|at| js::text(&side[at..]));
  let wrap = from_break(ahead).or_else(|| from_break(behind)).unwrap_or_else(|| " ".to_owned());

  Swap {
    from: dash.left + 1,
    to: dash.right,
    text: format!("{before}{wrap}{after}"),
  }
}

// Whether every bracket in a stretch closes inside it.
fn balanced(stretch: &[char]) -> bool {
  let mut depth = 0i64;

  for character in stretch {
    if opener(Some(character)) {
      depth += 1;
    } else if closer(Some(character)) {
      depth -= 1;

      if depth < 0 {
        return false;
      }
    }
  }

  depth == 0
}

fn piece_without_dashes(text: &str) -> String {
  if !any_dash(text) {
    return text.to_owned();
  }

  let piece = chars(text);
  let kept = kept_spans(&piece);
  let in_kept = |at: usize| kept.iter().any(|(start, end)| at >= *start && at < *end);
  let ends: Vec<usize> = (0..piece.len())
    .filter(|at| matches!(piece[*at], '.' | '!' | '?' | '\u{3002}'))
    .filter(|at| AFTER_SENTENCE.is_match(&js::text(&piece[at + 1..])))
    .filter(|at| !in_kept(*at))
    .collect();
  let mut dashes = vec![];

  for (start, end) in dash_runs(&piece) {
    if in_kept(start) {
      continue;
    }

    let mut left = start as i64 - 1;
    let mut right = end;

    while left >= 0 && is_space(piece.get(left as usize)) {
      left -= 1;
    }

    while right < piece.len() && is_space(piece.get(right)) {
      right += 1;
    }

    if left < 0 || right >= piece.len() {
      continue;
    }

    let left = left as usize;

    if piece[left] == '|' || piece[right] == '|' {
      continue;
    }

    // A dash a bracket or a quote holds is the character named, not a pause.
    if (opener(piece.get(left)) || piece[left] == '"') && (closer(piece.get(right)) || piece[right] == '"')
    {
      continue;
    }

    let line_start = js::rfind(&piece, &['\n'], left).map_or(0, |at| at + 1);

    if LIST_MARK.is_match(&js::text(&piece[line_start..=left])) {
      continue;
    }

    dashes.push(Dash {
      start,
      end,
      left,
      right,
      sentence: ends.iter().filter(|at| **at < start).count(),
    });
  }

  let mut swaps = vec![];
  let mut prose = vec![];

  for dash in dashes {
    let before = piece.get(dash.left);

    if dash.left + 1 == dash.start
      && dash.right == dash.end
      && is_digit(before)
      && is_digit(piece.get(dash.right))
    {
      swaps.push(Swap {
        from: dash.start,
        to: dash.end,
        text: "~".to_owned(),
      });
    } else if pause(before) {
      swaps.push(swap_at(&piece, &dash, "", ""));
    } else if opener(before) {
      swaps.push(Swap {
        from: dash.left + 1,
        to: dash.right,
        text: String::new(),
      });
    } else {
      prose.push(dash);
    }
  }

  let mut at = 0;

  while at < prose.len() {
    let dash = &prose[at];

    // Two dashes set off an aside only where no bracket opens or closes between them without the other.
    if let Some(next) = prose.get(at + 1)
      && next.sentence == dash.sentence
      && balanced(&piece[dash.right..=next.left])
    {
      let bracketed = piece[dash.right..=next.left].iter().any(aside_pause);
      // Where the sentence already pauses right after the aside, its own mark closes it.
      let closing = if pause(piece.get(next.right)) {
        Swap {
          from: next.left + 1,
          to: next.right,
          text: if bracketed { ")" } else { "" }.to_owned(),
        }
      } else {
        swap_at(&piece, next, if bracketed { ")" } else { "," }, "")
      };

      swaps.push(if bracketed { swap_at(&piece, dash, "", "(") } else { swap_at(&piece, dash, ",", "") });
      swaps.push(closing);
      at += 2;
      continue;
    }

    let line_start = js::rfind(&piece, &['\n'], dash.start).map_or(0, |at| at + 1);
    let label = LABEL
      .find(&js::text(&piece[line_start..]))
      .map(|found| found.as_str().chars().count() - 1);
    let opening = (if dash.sentence == 0 { 0 } else { ends[dash.sentence - 1] + 1 })
      .max(label.map_or(0, |label| line_start + label));
    let colon_before = (opening..dash.start).any(|at| {
      piece[at] == ':'
        && !in_kept(at)
        && !(at > 0 && is_digit(piece.get(at - 1)) && is_digit(piece.get(at + 1)))
    }) || swaps
      .iter()
      .any(|swap| swap.text.starts_with(':') && swap.from >= opening && swap.from < dash.start);

    swaps.push(swap_at(&piece, dash, if colon_before { "," } else { ":" }, ""));
    at += 1;
  }

  swaps.sort_by_key(|swap| swap.from);

  let mut written = String::new();
  let mut from = 0;

  for swap in swaps {
    written += &js::text(&piece[from.min(swap.from)..swap.from]);
    written += &swap.text;
    from = swap.to;
  }

  written + &js::text(&piece[from.min(piece.len())..])
}

/// A text with every dash left between words replaced; `wrapped` reads a paragraph or a list item of a
/// document wrapped at a fixed width as one piece, where a chat answer's every line is one.
pub fn without_dashes(text: &str, wrapped: bool) -> String {
  if !any_dash(text) {
    return text.to_owned();
  }

  let mut pieces: Vec<(Vec<&str>, bool)> = vec![];
  let mut fenced = false;

  for line in text.split('\n') {
    let fence = FENCE.is_match(line);
    let prose = !fence && !fenced && !QUOTE.is_match(line) && !js::trim(line).is_empty();

    if fence {
      fenced = !fenced;
    }

    match pieces.last_mut() {
      Some((lines, true))
        if wrapped && prose && !BLOCK_START.is_match(line) && !ONE_LINE_BLOCK.is_match(lines[0]) =>
      {
        lines.push(line);
      }
      _ => pieces.push((vec![line], prose)),
    }
  }

  pieces
    .iter()
    .map(|(lines, prose)| if *prose { piece_without_dashes(&lines.join("\n")) } else { lines.join("\n") })
    .collect::<Vec<_>>()
    .join("\n")
}
