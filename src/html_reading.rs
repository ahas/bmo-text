//! An HTML file read for what it says, the twin of the package's `html-reading`, so a document can be written
//! out of it. A page handed over as a file is often mostly pictures written into it as data addresses, which a
//! reader could never open whole, so the reading keeps the words in their order and takes every such picture
//! out as a file of its own, named where it stood.
//!
//! The reading is structural and nothing more: headings keep their level as markdown headings, a list item is a
//! dash, every block starts a line, and scripts and styles are not words. It does not run the page. It is read
//! with lol-html, the streaming rewriter the package's runtime reads with, so both see the same elements, end
//! tags and text.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::LazyLock;

use base64::Engine;
use base64::alphabet::STANDARD;
use base64::engine::{DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig};
use lol_html::html_content::{Element, TextChunk};
use lol_html::{HtmlRewriter, Settings, doc_text, element};
use regex::Regex;

use crate::answer_cap::{MAX_ANSWER_CHARS, Said, cap_items, cap_value};
use crate::js::{self, SPACE};

/// One picture a page carried inside itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HtmlPicture {
  /// The name it stands under in the reading, and the one it is kept by.
  pub name: String,
  pub media_type: String,
  pub bytes: Vec<u8>,
}

/// What a page says.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HtmlReading {
  pub title: Option<String>,
  /// What the page says of itself in its `description` meta, where it says anything.
  pub description: Option<String>,
  /// The page's words, a block to a line, with pictures as markdown pictures where they stood.
  pub lines: Vec<String>,
  /// Every picture the page carried inside itself, in the order they stood.
  pub pictures: Vec<HtmlPicture>,
}

const BLOCKS: [&str; 31] = [
  "address",
  "article",
  "aside",
  "blockquote",
  "br",
  "button",
  "dd",
  "details",
  "div",
  "dl",
  "dt",
  "figcaption",
  "figure",
  "footer",
  "form",
  "header",
  "hr",
  "li",
  "main",
  "nav",
  "ol",
  "p",
  "pre",
  "section",
  "small",
  "summary",
  "table",
  "td",
  "th",
  "tr",
  "ul",
];
const UNREAD: [&str; 6] = ["script", "style", "noscript", "template", "svg", "head"];

static HEADING: LazyLock<Regex> = LazyLock::new(|| Regex::new("^h([1-6])$").expect("a heading tag"));
static DESCRIPTION_META: LazyLock<Regex> =
  LazyLock::new(|| Regex::new("(?i)^(?:description|og:description)$").expect("a description meta"));
static DATA_PICTURE: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"(?s)^data:(image/[A-Za-z0-9_.+-]+)(;base64)?,(.*)$").expect("a picture written as data")
});
static SPACES: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(&format!("[{SPACE}]+")).expect("a run of space"));

// Base64 as the package's runtime reads it: either alphabet, padding or none, and the bits a short end leaves
// over kept rather than refused.
const LENIENT: GeneralPurpose = GeneralPurpose::new(
  &STANDARD,
  GeneralPurposeConfig::new()
    .with_decode_padding_mode(DecodePaddingMode::RequireNone)
    .with_decode_allow_trailing_bits(true),
);

fn extension(media_type: &str) -> String {
  match media_type {
    "image/jpeg" => ".jpg".to_owned(),
    "image/png" => ".png".to_owned(),
    "image/gif" => ".gif".to_owned(),
    "image/webp" => ".webp".to_owned(),
    "image/avif" => ".avif".to_owned(),
    "image/svg+xml" => ".svg".to_owned(),
    other => format!(".{}", other[6..].split('+').next().unwrap_or("")),
  }
}

// A base64 payload's bytes as the package's runtime decodes them: read up to the first padding mark, the
// characters of neither alphabet passed over, and a last lone character dropped.
fn base64_bytes(payload: &str) -> Vec<u8> {
  let mut kept: String = payload
    .chars()
    .take_while(|character| *character != '=')
    .filter_map(|character| match character {
      '-' => Some('+'),
      '_' => Some('/'),
      'A'..='Z' | 'a'..='z' | '0'..='9' | '+' | '/' => Some(character),
      _ => None,
    })
    .collect();

  if kept.len() % 4 == 1 {
    kept.pop();
  }

  LENIENT.decode(kept).unwrap_or_default()
}

// What the package's decoding of a text and its spaces gives: the entities read, every run of JavaScript's white
// space one space, the ends trimmed.
fn words(raw: &str) -> String {
  js::trim(&SPACES.replace_all(&htmlize::unescape(raw), " ")).to_owned()
}

// The reading as it is built, shared by the handlers of the one pass over the page.
#[derive(Default)]
struct Reading {
  done: HtmlReading,
  line: String,
  prefix: String,
  unread: usize,
  title: Option<String>,
  in_title: bool,
}

impl Reading {
  fn flush(&mut self) {
    let words = words(&self.line);

    self.line.clear();

    if !words.is_empty() {
      self.done.lines.push(format!("{}{words}", self.prefix));
      self.prefix.clear();
    }
  }

  fn picture(&mut self, source: &str, alt: &str, stem: &str) -> String {
    let Some(data) = DATA_PICTURE.captures(source) else {
      return format!("![{alt}]({source})");
    };
    let media_type = data[1].to_lowercase();
    let name = format!("{stem}-{:02}{}", self.done.pictures.len() + 1, extension(&media_type));
    let payload = &data[3];
    let bytes = if data.get(2).is_some() {
      base64_bytes(payload)
    } else {
      js::decode_uri_component(payload).unwrap_or_else(|| payload.to_owned()).into_bytes()
    };

    self.done.pictures.push(HtmlPicture {
      name: name.clone(),
      media_type,
      bytes,
    });

    format!("![{alt}]({name})")
  }
}

// What one element does to the reading.
fn on_element(reading: &Rc<RefCell<Reading>>, element: &mut Element, stem: &str) {
  let tag = element.tag_name().to_lowercase();
  let mut state = reading.borrow_mut();

  if tag == "meta" && state.done.description.is_none() {
    let kind = element
      .get_attribute("name")
      .or_else(|| element.get_attribute("property"))
      .unwrap_or_default();

    if DESCRIPTION_META.is_match(&kind) {
      let content = words(&element.get_attribute("content").unwrap_or_default());

      if !content.is_empty() {
        state.done.description = Some(content);
      }

      return;
    }
  }

  // An element that cannot hold content, such as a self-closing one in SVG, has no end tag to wait for.
  if tag == "title" {
    if let Some(handlers) = element.end_tag_handlers() {
      let reading = Rc::clone(reading);

      state.in_title = true;
      handlers.push(Box::new(move |_| {
        reading.borrow_mut().in_title = false;
        Ok(())
      }));
    }

    return;
  }

  if UNREAD.contains(&tag.as_str()) {
    if let Some(handlers) = element.end_tag_handlers() {
      let reading = Rc::clone(reading);

      state.unread += 1;
      handlers.push(Box::new(move |_| {
        reading.borrow_mut().unread -= 1;
        Ok(())
      }));
    }

    return;
  }

  if state.unread > 0 {
    return;
  }

  // Words in two neighbouring elements are two words, however the page set them side by side.
  state.line.push(' ');

  if tag == "img" {
    state.flush();

    let source = element.get_attribute("src").unwrap_or_default();
    let alt = htmlize::unescape(element.get_attribute("alt").unwrap_or_default()).into_owned();
    let line = state.picture(&source, &alt, stem);

    state.done.lines.push(line);
    return;
  }

  let heading = HEADING.captures(&tag).map(|found| found[1].parse::<usize>().expect("a heading level"));

  if heading.is_some() || BLOCKS.contains(&tag.as_str()) {
    state.flush();

    // A block inside a heading or a list item keeps the mark it stands under.
    if let Some(level) = heading {
      state.prefix = format!("{} ", "#".repeat(level));
    }

    if tag == "li" {
      "- ".clone_into(&mut state.prefix);
    }

    let self_closing = element.is_self_closing();

    if !self_closing
      && tag != "br"
      && tag != "hr"
      && let Some(handlers) = element.end_tag_handlers()
    {
      let reading = Rc::clone(reading);

      handlers.push(Box::new(move |_| {
        let mut state = reading.borrow_mut();

        if state.unread == 0 {
          state.flush();
        }

        Ok(())
      }));
    }
  }
}

fn on_text(reading: &Rc<RefCell<Reading>>, chunk: &TextChunk) {
  let mut state = reading.borrow_mut();

  if state.in_title {
    state.title.get_or_insert_with(String::new).push_str(chunk.as_str());
  } else if state.unread == 0 {
    state.line.push_str(chunk.as_str());
  }
}

/// The reading of one page. `stem` is what the pictures taken out of it are named after, numbered in the order
/// they stood. A picture whose percent escapes are not well formed keeps its payload's bytes as they stand.
pub fn read_html(html: &str, stem: &str) -> HtmlReading {
  let reading = Rc::new(RefCell::new(Reading::default()));
  let on_elements = Rc::clone(&reading);
  let on_texts = Rc::clone(&reading);
  let settings = Settings::new()
    .append_element_content_handler(element!("*", move |element| {
      on_element(&on_elements, element, stem);
      Ok(())
    }))
    .append_document_content_handler(doc_text!(move |chunk| {
      on_text(&on_texts, chunk);
      Ok(())
    }))
    // A page is read as far as it goes, never refused for markup a parser could read two ways.
    .with_strict(false);
  let mut rewriter = HtmlRewriter::new(settings, |_: &[u8]| {});

  // The handlers never fail and no memory limit is set, so neither step has an error to give.
  let _ = rewriter.write(html.as_bytes());
  let _ = rewriter.end();

  let mut state = std::mem::take(&mut *reading.borrow_mut());

  state.flush();
  state.done.title = state.title.as_deref().map(words).filter(|title| !title.is_empty());
  state.done
}

/// One page of a page's reading: its title, the lines from the one asked for that fit under the answer ceiling,
/// and whether more are left after them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HtmlPage {
  pub title: Option<String>,
  pub lines: Vec<String>,
  pub more: bool,
}

/// The reading of one page from its line `from`, counted from nought: as many lines as fit under
/// `MAX_ANSWER_CHARS` joined a line apart, and at least one, cut at the ceiling where it is over it alone, so
/// each page moves the reading on. `stem` names the pictures as `read_html` names them.
pub fn read_html_page(html: &str, stem: &str, from: usize) -> HtmlPage {
  let reading = read_html(html, stem);
  let rest = &reading.lines[from.min(reading.lines.len())..];
  let capped =
    cap_items(rest, |lines| lines.join("\n"), &Said::new("lines", "ask again from the next line."));
  let lines = if capped.kept.is_empty() {
    rest.first().map(|line| vec![cap_value(line, MAX_ANSWER_CHARS)]).unwrap_or_default()
  } else {
    capped.kept
  };

  HtmlPage {
    title: reading.title,
    more: lines.len() < rest.len(),
    lines,
  }
}
