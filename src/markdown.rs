//! An agent writes markdown and Telegram reads HTML; this is the one place the first becomes the second.
//! The markdown is read once, and the markup sent, the plain text held as its fallback and the blocks the fold
//! and the division work on all come out of that one reading, so none of them says what the others do not.
//!
//! Telegram shows a message line for line, so the reading is line-based rather than CommonMark's: a line
//! break is a line break, an indented line is text, and a heading, a list, a table or a rule — none of which
//! Telegram has a tag for — is laid flat into the text it holds. Only the tags the markup rulebook lists are
//! built, and a marker that pairs with nothing is text. No maintained markdown library reads a text that
//! way, and the TypeScript reading (the package's `markdown`) is the one this one must match character for
//! character, which is why the reading is written here.

use std::sync::LazyLock;

use regex::Regex;

use crate::format::{
  LONG_TEXT_LENGTH, escape_attribute, escape_html, expandable_markup, is_linkable, markup_fault,
  visible_text,
};
use crate::js::{self, DOT, SPACE, chars, is_space, units, units_of, within};
use crate::web_address::{bare_address_at, glued_after_address, markdown_link_at};

/// What a formatting pair makes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pair {
  Bold,
  Italic,
  Strike,
  Spoiler,
}

/// One piece of a line: words, a code span, a formatting pair, or a link, each holding what it wraps.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Inline {
  Text(String),
  Code(String),
  Pair(Pair, Vec<Inline>),
  Link(String, Vec<Inline>),
}

pub type Line = Vec<Inline>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block {
  Lines(Vec<Line>),
  Pre {
    language: Option<String>,
    text: String,
  },
  Quote(Document),
}

/// A document is its blocks and, before each, the line breaks that stood between it and the one before, so
/// a message comes out spaced the way it was written. The first gap is always empty.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Document {
  pub blocks: Vec<Block>,
  pub gaps: Vec<String>,
}

/// A piece of markdown read: its document, its markup, its plain text, and whether the markup says
/// anything the plain text does not.
#[derive(Clone, Debug)]
pub struct Converted {
  pub source: String,
  pub document: Document,
  pub markup: String,
  pub plain: String,
  pub formatted: bool,
}

// ---------------------------------------------------------------------------------------------------------
// Inline

// The characters a backslash takes the meaning away from; any other backslash is a character of the text.
fn escapable(character: Option<&char>) -> bool {
  matches!(character, Some('`' | '*' | '~' | '|' | '[' | ']' | '\\'))
}

// What may not stand glued to the outside of a pair: a letter, a digit, an underscore or a slash makes the
// marker part of a word or a path.
fn glued_outside(character: Option<&char>) -> bool {
  matches!(character, Some('A'..='Z' | 'a'..='z' | '0'..='9' | '_' | '/'))
}

const PAIRS: [(&str, Pair); 4] =
  [("**", Pair::Bold), ("~~", Pair::Strike), ("||", Pair::Spoiler), ("*", Pair::Italic)];

fn before(text: &[char], at: usize) -> Option<&char> {
  at.checked_sub(1).and_then(|at| text.get(at))
}

fn starts_with(text: &[char], at: usize, marker: &str) -> bool {
  let marker = chars(marker);

  text.len() >= at + marker.len() && text[at..at + marker.len()] == *marker
}

fn backtick_run(text: &[char], at: usize) -> usize {
  text[at..].iter().take_while(|character| **character == '`').count()
}

// Where the code span opening at `at` closes: a run of exactly as many backticks.
fn code_close(text: &[char], at: usize, run: usize) -> Option<usize> {
  let mut from = at + run;

  while from < text.len() {
    let next = from + text[from..].iter().position(|character| *character == '`')?;
    let length = backtick_run(text, next);

    if length == run {
      return Some(next);
    }

    from = next + length;
  }

  None
}

fn opens_pair(text: &[char], at: usize, marker: &str) -> bool {
  let character = marker.chars().next();
  let length = marker.chars().count();
  let ahead = before(text, at);
  let after = text.get(at + length);

  starts_with(text, at, marker)
    && !glued_outside(ahead)
    && ahead != character.as_ref()
    && after.is_some_and(|after| !is_space(*after))
    && after != character.as_ref()
}

fn closes_pair(text: &[char], at: usize, marker: &str) -> bool {
  let character = marker.chars().next();
  let length = marker.chars().count();
  let ahead = before(text, at);
  let after = text.get(at + length);

  starts_with(text, at, marker)
    && ahead.is_some_and(|ahead| !is_space(*ahead))
    && ahead != character.as_ref()
    && !glued_outside(after)
    && after != character.as_ref()
}

// Where the pair opening at `at` closes, looking past code spans and escapes, which hold no markers.
fn pair_close(text: &[char], at: usize, marker: &str) -> Option<usize> {
  let mut index = at + marker.chars().count() + 1;

  while index < text.len() {
    if text[index] == '\\' && escapable(text.get(index + 1)) {
      index += 2;
      continue;
    }

    if text[index] == '`' {
      let run = backtick_run(text, index);

      index = code_close(text, index, run).map_or(index + run, |close| close + run);
      continue;
    }

    if closes_pair(text, index, marker) {
      return Some(index);
    }

    index += 1;
  }

  None
}

static AUTOLINK: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(&format!(r"(?i)^<(https?://[^{SPACE}<>]+)>")).expect("an autolink"));

fn push_text(nodes: &mut Vec<Inline>, text: &str) {
  if text.is_empty() {
    return;
  }

  if let Some(Inline::Text(last)) = nodes.last_mut() {
    last.push_str(text);
  } else {
    nodes.push(Inline::Text(text.to_owned()));
  }
}

// Nodes put after others, words joining the words before them.
fn push_all(nodes: &mut Vec<Inline>, more: Vec<Inline>) {
  for node in more {
    match node {
      Inline::Text(text) => push_text(nodes, &text),
      node => nodes.push(node),
    }
  }
}

/// Where a construct stood in the line it was read from: a code span or a link, literal to its end, or a
/// formatting pair. A cut through one would leave both halves holding a marker with nothing to pair with.
#[derive(Clone, Copy, Debug)]
pub struct Span {
  pub start: usize,
  pub end: usize,
  pub literal: bool,
}

/// One line of markdown as inline nodes; nothing crosses a line break. `spans`, where given, collects where
/// each construct of the line stood.
pub fn parse_inline(text: &[char], mut spans: Option<&mut Vec<Span>>) -> Vec<Inline> {
  let mut nodes = vec![];
  let mut index = 0;
  let mut stood = |start: usize, end: usize, literal: bool| {
    if let Some(spans) = spans.as_deref_mut() {
      spans.push(Span {
        start,
        end,
        literal,
      });
    }
  };

  while index < text.len() {
    let character = text[index];

    if character == '\\' && escapable(text.get(index + 1)) {
      push_text(&mut nodes, &text[index + 1].to_string());
      index += 2;
      continue;
    }

    if character == '`' {
      let run = backtick_run(text, index);
      let Some(close) = code_close(text, index, run) else {
        push_text(&mut nodes, &"`".repeat(run));
        index += run;
        continue;
      };
      let mut inner = &text[index + run..close];

      if inner.len() > 1
        && inner[0] == ' '
        && inner[inner.len() - 1] == ' '
        && !js::trim(&js::text(inner)).is_empty()
      {
        inner = &inner[1..inner.len() - 1];
      }

      nodes.push(Inline::Code(js::text(inner)));
      stood(index, close + run, true);
      index = close + run;
      continue;
    }

    let image = character == '!' && text.get(index + 1) == Some(&'[');

    if (character == '[' || image)
      && let Some(found) = markdown_link_at(text, if image { index + 1 } else { index })
    {
      let label = if image {
        vec![Inline::Text(found.label.clone())]
      } else {
        parse_inline(&chars(&found.label), None)
      };
      let shown = if label.is_empty() { vec![Inline::Text(found.url.clone())] } else { label };

      if is_linkable(&found.url) {
        nodes.push(Inline::Link(found.url.clone(), shown));
      } else {
        push_all(&mut nodes, shown);
        push_text(&mut nodes, &format!(" {}", found.url));
      }

      stood(index, found.end, true);
      index = found.end;
      continue;
    }

    if character == '<'
      && let Some(found) = AUTOLINK.captures(&js::text(&text[index..]))
    {
      let length = found[0].chars().count();

      push_text(&mut nodes, &found[1]);
      stood(index, index + length, true);
      index += length;
      continue;
    }

    // A bare address stays as characters for Telegram to show as a link on its own, except where the
    // sentence runs on straight after it, which Telegram would read as part of the address.
    if (character == 'h' || character == 'H')
      && let Some(url) = bare_address_at(text, index)
    {
      let length = url.chars().count();

      if glued_after_address(text, index + length) {
        nodes.push(Inline::Link(url.clone(), vec![Inline::Text(url)]));
        stood(index, index + length, true);
        index += length;
        continue;
      }
    }

    if let Some((marker, pair)) = PAIRS.iter().find(|(marker, _)| opens_pair(text, index, marker)) {
      let length = marker.chars().count();

      if let Some(close) = pair_close(text, index, marker) {
        nodes.push(Inline::Pair(*pair, parse_inline(&text[index + length..close], None)));
        stood(index, close + length, false);
        index = close + length;
        continue;
      }
    }

    // A run of one marker character that opened nothing stays text as one piece.
    if matches!(character, '*' | '~' | '|') {
      let end = index + text[index..].iter().take_while(|one| **one == character).count();

      push_text(&mut nodes, &js::text(&text[index..end]));
      index = end;
      continue;
    }

    push_text(&mut nodes, &character.to_string());
    index += 1;
  }

  nodes
}

// ---------------------------------------------------------------------------------------------------------
// Blocks

static FENCE_OPEN: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(&format!(r"^ {{0,3}}(`{{3,}}|~{{3,}})[ \t]*([^{SPACE}`]*)[^`]*$")).expect("a fence")
});
static FENCE_CLOSE: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^ {0,3}(`{3,}|~{3,})[ \t]*$").expect("a fence's close"));
static LANGUAGE_NAME: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^[A-Za-z0-9_+#.-]{1,32}$").expect("a language's name"));
static HEADING: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(&format!(r"^ {{0,3}}#{{1,6}}(?:[ \t]+({DOT}*?))?(?:[ \t]+#+)?[ \t]*$")).expect("a heading")
});
static RULE: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"^ {0,3}(?:-(?:[ \t]*-){2,}|\*(?:[ \t]*\*){2,}|_(?:[ \t]*_){2,})[ \t]*$").expect("a rule")
});
static TABLE_RULE: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"^[ \t]*\|?[ \t]*:?-+:?[ \t]*(?:\|[ \t]*:?-+:?[ \t]*)*\|?[ \t]*$").expect("a table's rule")
});
static BULLET: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(&format!(r"^([ \t]*)[-*+][ \t]+({DOT}*)$")).expect("a bullet"));
static NUMBERED: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(&format!(r"^([ \t]*[0-9]{{1,3}}[.)][ \t]+)({DOT}*)$")).expect("a numbered line")
});
static QUOTE_LINE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^ {0,3}>").expect("a quoted line"));
static QUOTE_MARKS: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^ {0,3}(?:>[ \t]?)+").expect("quote marks"));

// The cells of a table row. A pipe is a cell border only outside a code span or a link, and unescaped.
fn table_cells(row: &str) -> Vec<String> {
  let mut body = js::trim(row);

  if let Some(rest) = body.strip_prefix('|') {
    body = rest;
  }

  if body.ends_with('|') && !body.ends_with("\\|") {
    body = &body[..body.len() - 1];
  }

  let body = chars(body);
  let mut spans = vec![];

  parse_inline(&body, Some(&mut spans));

  let mut cells = vec![];
  let mut from = 0;

  for index in 0..body.len() {
    let inside = spans.iter().any(|span| span.literal && span.start < index && index < span.end);

    if body[index] == '|' && before(&body, index) != Some(&'\\') && !inside {
      cells.push(js::text(&body[from..index]));
      from = index + 1;
    }
  }

  cells.push(js::text(&body[from..]));
  cells.iter().map(|cell| js::trim(cell).to_owned()).collect()
}

fn is_table_start(lines: &[String], at: usize) -> bool {
  let header = lines.get(at).map_or("", String::as_str);
  let rule = lines.get(at + 1).map_or("", String::as_str);

  header.contains('|') && rule.contains('|') && TABLE_RULE.is_match(rule)
}

fn table_row(row: &str) -> Line {
  let mut line = vec![];

  for (index, cell) in table_cells(row).iter().enumerate() {
    if index > 0 {
      push_text(&mut line, " \u{b7} ");
    }

    push_all(&mut line, parse_inline(&chars(cell), None));
  }

  line
}

fn prefixed(prefix: &str, rest: &str) -> Line {
  let mut line = vec![Inline::Text(prefix.to_owned())];

  push_all(&mut line, parse_inline(&chars(rest), None));
  line
}

// A line of text laid flat: a heading loses its marks, a bullet becomes •, a number stays as written. None
// is a line that is no text at all: a rule, or a heading with nothing in it.
fn flat_line(line: &str) -> Option<Line> {
  if RULE.is_match(line) {
    return None;
  }

  if let Some(heading) = HEADING.captures(line) {
    let words = heading.get(1).map_or("", |words| words.as_str());

    return (!js::trim(words).is_empty()).then(|| parse_inline(&chars(words), None));
  }

  if let Some(bullet) = BULLET.captures(line) {
    return Some(prefixed(&format!("{}\u{2022} ", &bullet[1]), &bullet[2]));
  }

  if let Some(numbered) = NUMBERED.captures(line) {
    return Some(prefixed(&numbered[1], &numbered[2]));
  }

  Some(parse_inline(&chars(line), None))
}

fn opener_of(found: &regex::Captures<'_>) -> String {
  found.get(1).map_or("```", |opener| opener.as_str()).to_owned()
}

// A fence's closing line for the run that opened it: the same character, at least as many of it.
fn closes(line: &str, opener: &str) -> bool {
  FENCE_CLOSE.captures(line).is_some_and(|closing| {
    let run = &closing[1];

    run.chars().next() == opener.chars().next() && run.chars().count() >= opener.chars().count()
  })
}

fn parse_blocks(lines: &[String], quotes: bool) -> Document {
  let mut document = Document::default();
  let mut current: Vec<Line> = vec![];
  // Line breaks owed before the next block: one ends the line before it, and every blank line adds one.
  let mut pending: usize = 0;
  let mut index = 0;

  fn push(document: &mut Document, pending: &mut usize, block: Block) {
    document.gaps.push(if document.blocks.is_empty() {
      String::new()
    } else {
      "\n".repeat((*pending).max(1))
    });
    document.blocks.push(block);
    *pending = 0;
  }

  let flush = |document: &mut Document, pending: &mut usize, current: &mut Vec<Line>| {
    if !current.is_empty() {
      push(document, pending, Block::Lines(std::mem::take(current)));
      *pending = 1;
    }
  };

  while index < lines.len() {
    let line = &lines[index];

    if js::trim(line).is_empty() {
      flush(&mut document, &mut pending, &mut current);
      pending += 1;
      index += 1;
      continue;
    }

    if let Some(fence) = FENCE_OPEN.captures(line) {
      flush(&mut document, &mut pending, &mut current);

      let opener = opener_of(&fence);
      let language = fence.get(2).map_or("", |language| language.as_str()).to_owned();
      let mut body: Vec<&str> = vec![];

      index += 1;

      while index < lines.len() && !closes(&lines[index], &opener) {
        body.push(&lines[index]);
        index += 1;
      }

      index += 1;

      // An empty code block is a tag around nothing, which Telegram has no use for.
      if !js::trim(&body.concat()).is_empty() {
        let language = LANGUAGE_NAME.is_match(&language).then_some(language);

        push(
          &mut document,
          &mut pending,
          Block::Pre {
            language,
            text: body.join("\n"),
          },
        );
        pending = 1;
      }

      continue;
    }

    if quotes && QUOTE_LINE.is_match(line) {
      flush(&mut document, &mut pending, &mut current);

      let mut inner = vec![];

      while index < lines.len() && QUOTE_LINE.is_match(&lines[index]) {
        inner.push(QUOTE_MARKS.replace(&lines[index], "").into_owned());
        index += 1;
      }

      let quoted = parse_blocks(&inner, false);

      if !quoted.blocks.is_empty() {
        push(&mut document, &mut pending, Block::Quote(quoted));
        pending = 1;
      }

      continue;
    }

    if is_table_start(lines, index) {
      current.push(table_row(line));
      index += 2;

      while index < lines.len() && lines[index].contains('|') && !js::trim(&lines[index]).is_empty() {
        current.push(table_row(&lines[index]));
        index += 1;
      }

      continue;
    }

    match flat_line(line) {
      Some(flat) => current.push(flat),
      // A rule standing between blank lines takes one of them with it.
      None if current.is_empty() => pending = pending.saturating_sub(1),
      None => {}
    }

    index += 1;
  }

  flush(&mut document, &mut pending, &mut current);
  document
}

pub fn parse_markdown(source: &str) -> Document {
  let lines: Vec<String> = source.replace("\r\n", "\n").split('\n').map(str::to_owned).collect();

  parse_blocks(&lines, true)
}

// ---------------------------------------------------------------------------------------------------------
// Rendering

// A formatting or a link around a leaf, told apart from every other by the number it was met as.
#[derive(Clone, Debug)]
struct Wrapper<'a> {
  id: usize,
  node: &'a Inline,
}

impl Wrapper<'_> {
  fn open(&self) -> String {
    match self.node {
      Inline::Link(url, _) => format!("<a href=\"{}\">", escape_attribute(url)),
      Inline::Pair(Pair::Spoiler, _) => "<tg-spoiler>".to_owned(),
      Inline::Pair(pair, _) => format!("<{}>", tag(*pair)),
      _ => String::new(),
    }
  }

  fn close(&self) -> String {
    match self.node {
      Inline::Link(..) => "</a>".to_owned(),
      Inline::Pair(Pair::Spoiler, _) => "</tg-spoiler>".to_owned(),
      Inline::Pair(pair, _) => format!("</{}>", tag(*pair)),
      _ => String::new(),
    }
  }
}

fn tag(pair: Pair) -> &'static str {
  match pair {
    Pair::Bold => "b",
    Pair::Italic => "i",
    Pair::Strike => "s",
    Pair::Spoiler => "tg-spoiler",
  }
}

// A code span inside a link keeps its letters and loses its tag, since a link may not hold one.
struct Leaf<'a> {
  code: bool,
  text: &'a str,
  wrappers: Vec<Wrapper<'a>>,
}

fn leaves<'a>(nodes: &'a [Inline], wrappers: &[Wrapper<'a>], next: &mut usize, into: &mut Vec<Leaf<'a>>) {
  for node in nodes {
    match node {
      Inline::Text(text) | Inline::Code(text) => {
        let in_link = wrappers.iter().any(|one| matches!(one.node, Inline::Link(..)));

        into.push(Leaf {
          code: matches!(node, Inline::Code(_)) && !in_link,
          text,
          wrappers: wrappers.to_vec(),
        });
      }
      Inline::Pair(_, children) | Inline::Link(_, children) => {
        let mut inner = wrappers.to_vec();

        *next += 1;
        inner.push(Wrapper { id: *next, node });
        leaves(children, &inner, next, into);
      }
    }
  }
}

/// A line's markup, built from its leaves with a stack of open tags. A code leaf closes every tag around it
/// and reopens them after, since Telegram refuses a code inside a bold; opening and closing by the stack keeps
/// every tag paired and nested.
pub fn render_inline(nodes: &[Inline]) -> String {
  let mut found = vec![];
  let mut open: Vec<Wrapper<'_>> = vec![];
  let mut out = String::new();

  leaves(nodes, &[], &mut 0, &mut found);

  for leaf in &found {
    if leaf.text.is_empty() {
      continue;
    }

    let wanted: &[Wrapper<'_>] = if leaf.code { &[] } else { &leaf.wrappers };
    let shared = open.iter().zip(wanted).take_while(|(one, other)| one.id == other.id).count();

    while open.len() > shared {
      out += &open.pop().map(|wrapper| wrapper.close()).unwrap_or_default();
    }

    for wrapper in &wanted[shared..] {
      out += &wrapper.open();
      open.push(wrapper.clone());
    }

    if leaf.code {
      out += &format!("<code>{}</code>", escape_html(leaf.text));
    } else {
      out += &escape_html(leaf.text);
    }
  }

  while let Some(wrapper) = open.pop() {
    out += &wrapper.close();
  }

  out
}

pub fn plain_inline(nodes: &[Inline]) -> String {
  nodes
    .iter()
    .map(|node| match node {
      Inline::Text(text) | Inline::Code(text) => text.clone(),
      Inline::Link(url, children) => {
        let label = plain_inline(children);

        if label == *url { label } else { format!("{label} ({url})") }
      }
      Inline::Pair(_, children) => plain_inline(children),
    })
    .collect()
}

// `quoted` is false inside the folded part: an expandable quote cannot hold a quote, so one there stands as
// its own lines.
fn render_block(block: &Block, quoted: bool) -> String {
  match block {
    Block::Lines(lines) => lines.iter().map(|line| render_inline(line)).collect::<Vec<_>>().join("\n"),
    Block::Pre {
      language: None,
      text,
    } => format!("<pre>{}</pre>", escape_html(text)),
    Block::Pre {
      language: Some(language),
      text,
    } => {
      format!("<pre><code class=\"language-{language}\">{}</code></pre>", escape_html(text))
    }
    Block::Quote(inner) => {
      let inner = render_blocks(&inner.blocks, &inner.gaps, true);

      if quoted { format!("<blockquote>{inner}</blockquote>") } else { inner }
    }
  }
}

fn gap(gaps: &[String], index: usize) -> &str {
  gaps.get(index).map_or("", String::as_str)
}

fn render_blocks(blocks: &[Block], gaps: &[String], quoted: bool) -> String {
  blocks
    .iter()
    .enumerate()
    .map(|(index, block)| format!("{}{}", gap(gaps, index), render_block(block, quoted)))
    .collect()
}

fn plain_block(block: &Block) -> String {
  match block {
    Block::Lines(lines) => lines.iter().map(|line| plain_inline(line)).collect::<Vec<_>>().join("\n"),
    Block::Pre { text, .. } => text.clone(),
    Block::Quote(inner) => plain_blocks(&inner.blocks, &inner.gaps),
  }
}

fn plain_blocks(blocks: &[Block], gaps: &[String]) -> String {
  blocks
    .iter()
    .enumerate()
    .map(|(index, block)| format!("{}{}", gap(gaps, index), plain_block(block)))
    .collect()
}

// ---------------------------------------------------------------------------------------------------------
// The one-bold rule

// A line's content once a leading bullet, number or circled number and the spaces around it are set aside.
static ITEM_PREFIX: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new("^[ \t]*(?:\u{2022}|[0-9]{1,3}[.)]|[\u{2460}-\u{2473}])?[ \t]*").expect("an item's prefix")
});

// Whether the bold standing at `at` of a line is the whole of it but an item's prefix.
fn is_whole_line(line: &Line, at: usize) -> bool {
  let mut others = String::new();

  for (index, node) in line.iter().enumerate() {
    match node {
      _ if index == at => {}
      Inline::Text(text) => others.push_str(text),
      _ => return false,
    }
  }

  js::trim(&ITEM_PREFIX.replace(&others, "")).is_empty()
}

fn lines_of(blocks: &[Block]) -> Vec<&Line> {
  blocks
    .iter()
    .flat_map(|block| match block {
      Block::Lines(lines) => lines.iter().collect(),
      Block::Quote(inner) => lines_of(&inner.blocks),
      Block::Pre { .. } => vec![],
    })
    .collect()
}

fn each_line_mut(blocks: &mut [Block], visit: &mut impl FnMut(&mut Line)) {
  for block in blocks {
    match block {
      Block::Lines(lines) => lines.iter_mut().for_each(&mut *visit),
      Block::Quote(inner) => each_line_mut(&mut inner.blocks, visit),
      Block::Pre { .. } => {}
    }
  }
}

// Whether each bold of some nodes is the whole of its line, in order, not looking inside a bold.
fn bolds_in(nodes: &[Inline], line: Option<&Line>, into: &mut Vec<bool>) {
  for (at, node) in nodes.iter().enumerate() {
    match node {
      Inline::Pair(Pair::Bold, _) => into.push(line.is_some_and(|line| is_whole_line(line, at))),
      Inline::Pair(_, children) | Inline::Link(_, children) => bolds_in(children, None, into),
      _ => {}
    }
  }
}

// Every bold after the first, met in order and not looking inside a bold, replaced by what it holds.
fn unbold(nodes: &mut Vec<Inline>, met: &mut usize) {
  let mut kept = Vec::with_capacity(nodes.len());

  for node in std::mem::take(nodes) {
    match node {
      Inline::Pair(Pair::Bold, children) => {
        *met += 1;

        if *met > 1 {
          kept.extend(children);
        } else {
          kept.push(Inline::Pair(Pair::Bold, children));
        }
      }
      Inline::Pair(pair, mut children) => {
        unbold(&mut children, met);
        kept.push(Inline::Pair(pair, children));
      }
      Inline::Link(url, mut children) => {
        unbold(&mut children, met);
        kept.push(Inline::Link(url, children));
      }
      node => kept.push(node),
    }
  }

  *nodes = kept;
}

// Bold is one per message, but for the item names of a list: several survive only where each is a whole
// line, and otherwise the first stands alone and the rest keep their words without the bold.
fn limit_bold(document: &mut Document) {
  let mut bolds = vec![];

  for line in lines_of(&document.blocks) {
    bolds_in(line, Some(line), &mut bolds);
  }

  if bolds.len() <= 1 || bolds.iter().all(|whole| *whole) {
    return;
  }

  let mut met = 0;

  each_line_mut(&mut document.blocks, &mut |line| unbold(line, &mut met));
}

// ---------------------------------------------------------------------------------------------------------
// Conversion

pub fn convert_markdown(source: &str) -> Converted {
  let mut document = parse_markdown(source);

  limit_bold(&mut document);

  let markup = render_blocks(&document.blocks, &document.gaps, true);
  let plain = plain_blocks(&document.blocks, &document.gaps);
  let formatted = markup != escape_html(&plain);

  Converted {
    source: source.to_owned(),
    document,
    markup,
    plain,
    formatted,
  }
}

// ---------------------------------------------------------------------------------------------------------
// The fold

const FOLD_MAX_OPENING_LENGTH: usize = 400;
const FOLD_MIN_HIDDEN_LENGTH: usize = 80;
const FOLD_MIN_HIDDEN_LINES: usize = 3;
// A last block this short is the few lines that tie the message together and never folds.
const FOLD_MAX_CLOSING_LENGTH: usize = 200;
const FOLD_MAX_CLOSING_LINES: usize = 3;
const FOLD_MIN_LINES: usize = 8;

// A sentence that ends, which a conclusion is and the last line of a list is not.
static FINISHED_SENTENCE: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new("[.!?\u{2026}][\"'\u{201d}\u{2019})\\]\u{300d}\u{300f}]?$").expect("a finished sentence")
});

fn line_count(text: &str) -> usize {
  text.split('\n').count()
}

fn is_conclusion(text: &str) -> bool {
  units(text) <= FOLD_MAX_CLOSING_LENGTH && line_count(text) <= FOLD_MAX_CLOSING_LINES
}

/// Where a fold falls, decided on blocks: the opening and the closing stay open and the hidden blocks go
/// behind the quote. A code block or a quote is one block, so the fold never cuts through one.
#[derive(Clone, Debug)]
pub struct Fold {
  pub opening: Option<Block>,
  pub opening_gap: String,
  pub hidden: Vec<Block>,
  pub hidden_gaps: Vec<String>,
  pub closing: Option<Block>,
  pub closing_gap: String,
}

pub fn fold_blocks(document: &Document) -> Option<Fold> {
  let mut body = document.blocks.clone();
  let mut body_gaps = document.gaps.clone();
  let mut closing = None;
  let mut closing_gap = String::new();

  match body.last().cloned() {
    Some(last @ Block::Lines(_)) if body.len() > 1 && is_conclusion(&plain_block(&last)) => {
      closing_gap = body_gaps.last().cloned().unwrap_or_else(|| "\n\n".to_owned());
      closing = Some(last);
      body.pop();
      body_gaps.pop();
    }
    Some(Block::Lines(lines)) if lines.len() > 1 => {
      let (tail, rest) = lines.split_last().expect("more than one line");
      let tail_text = js::trim(&plain_inline(tail)).to_owned();
      let rest = Block::Lines(rest.to_vec());

      if !js::trim(&plain_block(&rest)).is_empty()
        && is_conclusion(&tail_text)
        && FINISHED_SENTENCE.is_match(&tail_text)
      {
        closing = Some(Block::Lines(vec![tail.clone()]));
        closing_gap = "\n".to_owned();
        body.pop();
        body.push(rest);
      }
    }
    _ => {}
  }

  let opening = body
    .first()
    .filter(|first| body.len() > 1 && units(&plain_block(first)) <= FOLD_MAX_OPENING_LENGTH)
    .cloned();
  let hidden: Vec<Block> = if opening.is_none() { body.clone() } else { body[1..].to_vec() };
  let skipped = if opening.is_none() { 1 } else { 2 };
  let hidden_gaps: Vec<String> =
    std::iter::once(String::new()).chain(body_gaps.iter().skip(skipped).cloned()).collect();
  let hidden_text = plain_blocks(&hidden, &hidden_gaps);

  if hidden.is_empty()
    || js::trim(&hidden_text).is_empty()
    || (units(&hidden_text) < FOLD_MIN_HIDDEN_LENGTH && line_count(&hidden_text) < FOLD_MIN_HIDDEN_LINES)
  {
    return None;
  }

  Some(Fold {
    opening,
    opening_gap: body_gaps.get(1).cloned().unwrap_or_else(|| "\n\n".to_owned()),
    hidden,
    hidden_gaps,
    closing,
    closing_gap,
  })
}

/// The message folded, or none where there is too little to hide for a fold to help.
pub fn folded_markup(converted: &Converted) -> Option<String> {
  let fold = fold_blocks(&converted.document)?;
  let opened = fold
    .opening
    .as_ref()
    .map_or_else(String::new, |opening| render_block(opening, true) + &fold.opening_gap);
  let quoted = expandable_markup(&render_blocks(&fold.hidden, &fold.hidden_gaps, false));
  let closed = fold
    .closing
    .as_ref()
    .map_or_else(String::new, |closing| fold.closing_gap.clone() + &render_block(closing, true));

  Some(opened + &quoted + &closed)
}

/// What a send that asks no gate puts up: the code's own fold where the message is long enough for one, the
/// converted markup where it carries formatting, and none — the plain text — otherwise. Markup the structure
/// check refuses is never it.
pub fn code_markup(converted: &Converted) -> Option<String> {
  let long = units(&converted.plain) > LONG_TEXT_LENGTH || line_count(&converted.plain) > FOLD_MIN_LINES;
  let folded = if long { folded_markup(converted) } else { None };
  let chosen = folded.or_else(|| converted.formatted.then(|| converted.markup.clone()));

  chosen.filter(|chosen| markup_fault(chosen).is_none())
}

// ---------------------------------------------------------------------------------------------------------
// Division

/// How long a piece of markdown is to Telegram: the longer of what its markup shows and its plain text.
pub fn sent_length(source: &[char]) -> usize {
  let converted = convert_markdown(&js::text(source));

  units(&visible_text(&converted.markup)).max(units(&converted.plain))
}

// Whether the source up to `at` leaves a fence open, and its opening line if so.
fn open_fence(source: &[char], at: usize) -> Option<String> {
  let mut open: Option<String> = None;
  let mut marker = String::new();

  for line in js::text(&source[..at]).split('\n') {
    match &open {
      None => {
        if let Some(found) = FENCE_OPEN.captures(line) {
          open = Some(js::trim_end(line).to_owned());
          marker = opener_of(&found);
        }
      }
      Some(_) => {
        if closes(line, &marker) {
          open = None;
        }
      }
    }
  }

  open
}

// Where a part ends inside a window of so many UTF-16 units: after the last blank line, else the last line
// break, else the last space outside every construct of the first line, else the last space, else the
// window's end.
fn cut_point(text: &[char], window: usize) -> usize {
  let head = &text[..within(text, window)];

  for pause in ["\n\n", "\n"] {
    let pause = chars(pause);

    if let Some(at) = js::rfind(head, &pause, head.len()).filter(|at| *at > 0) {
      return at + pause.len();
    }
  }

  let first_line = text.iter().position(|character| *character == '\n').unwrap_or(text.len());
  let mut spans = vec![];

  parse_inline(&text[..first_line], Some(&mut spans));

  let space = [' '];
  let mut at = js::rfind(head, &space, head.len());

  while let Some(found) = at.filter(|at| *at > 0) {
    if !spans.iter().any(|span| span.start < found && found < span.end) {
      return found + 1;
    }

    at = js::rfind(head, &space, found - 1);
  }

  match js::rfind(head, &space, head.len()) {
    Some(loose) if loose > 0 => loose + 1,
    _ => head.len(),
  }
}

static FENCE_RUN: LazyLock<Regex> = LazyLock::new(|| Regex::new("`{3,}|~{3,}").expect("a fence's run"));

/// A longer text divided into consecutive messages the way a reader pauses, each part a whole piece of
/// markdown: a code block cut in two is closed at the end of one part and opened again, with its language,
/// at the start of the next. A part is measured as Telegram will count it once converted, and cut shorter
/// where that is still too long.
pub fn markdown_parts(text: &str, limit: usize) -> Vec<String> {
  let mut parts = vec![];
  let mut rest = chars(text);

  while !rest.is_empty() && sent_length(&rest) > limit {
    let mut window = units_of(&rest).min(limit);
    let mut part;
    let mut next;

    loop {
      let cut = cut_point(&rest, window).max(1);
      let fence = open_fence(&rest, cut);

      part = rest[..cut].to_vec();
      next = rest[cut..].to_vec();

      if let Some(fence) = fence {
        let marker = FENCE_RUN.find(&fence).map_or("```", |found| found.as_str());

        if part.last() != Some(&'\n') {
          part.push('\n');
        }

        part.extend(marker.chars());
        next = [chars(&fence), vec!['\n'], next].concat();
      }

      if sent_length(&part) <= limit || window <= 1 {
        break;
      }

      window = (window as f64 * 0.9).floor() as usize;
    }

    // A division that gave back as much as it took would never end, so the window is cut as it stands.
    if units_of(&next) >= units_of(&rest) {
      let cut = within(&rest, limit);

      part = rest[..cut].to_vec();
      next = rest[cut..].to_vec();
    }

    parts.push(js::text(&part));
    rest = next;
  }

  if !rest.is_empty() || parts.is_empty() {
    parts.push(js::text(&rest));
  }

  parts
}
