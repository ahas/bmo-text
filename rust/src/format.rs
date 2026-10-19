//! Telegram's HTML parse mode as a chat service uses it: the escapes, the one quote a fold is made of, and
//! the structure check every markup is read by before it is sent. Telegram refuses a whole message over a tag
//! it does not know, one that never closes or a nesting it does not take, so the check pairs the tags
//! itself rather than leaving it to a model.

use std::sync::LazyLock;

use regex::Regex;

use crate::js::SPACE;

/// In HTML parse mode only these three characters carry meaning outside a tag.
pub fn escape_html(text: &str) -> String {
  text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// An attribute value is delimited by double quotes as well, so it needs one more.
pub fn escape_attribute(value: &str) -> String {
  escape_html(value).replace('"', "&quot;")
}

// Telegram takes only these schemes in an href; anything else fails the whole message.
static LINKABLE: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(&format!(r"(?i)^https?://[^{SPACE}]+$")).expect("a linkable address"));

pub fn is_linkable(url: &str) -> bool {
  LINKABLE.is_match(url)
}

static TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new("<[^>]*>").expect("a tag"));

/// What a reader is left with once the tags come out and the three escapes go back.
pub fn visible_text(markup: &str) -> String {
  TAG
    .replace_all(markup, "")
    .replace("&lt;", "<")
    .replace("&gt;", ">")
    .replace("&amp;", "&")
}

/// Markup already built from escaped text, wrapped in the quote a fold is.
pub fn expandable_markup(markup: &str) -> String {
  format!("<blockquote expandable>{markup}</blockquote>")
}

pub fn expandable(text: &str) -> String {
  expandable_markup(&escape_html(text))
}

/// Past this many UTF-16 units a message is long enough that hiding the bulk of it helps.
pub const LONG_TEXT_LENGTH: usize = 600;

static TAG_AT: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(&format!(r"^<(/?)([a-zA-Z][A-Za-z0-9_-]*)((?:[{SPACE}][^>]*)?)>")).expect("a tag")
});
static ENTITY_AT: LazyLock<Regex> =
  LazyLock::new(|| Regex::new("(?i)^&(?:lt|gt|amp|quot|#[0-9]+|#x[0-9a-f]+);").expect("an entity"));
static SPOILER_CLASS: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(&format!(r#"(?:^|[{SPACE}])class="tg-spoiler"(?:[{SPACE}]|$)"#)).expect("a spoiler's class")
});

// Bold, italic, underline, strikethrough and spoiler, which hold one another freely.
const FORMATTING: [&str; 11] =
  ["b", "strong", "i", "em", "u", "ins", "s", "strike", "del", "tg-spoiler", "span"];
// A literal: no formatting goes inside one and none holds one, but for a code directly inside a pre.
const LITERAL: [&str; 2] = ["code", "pre"];
// The rest the markup rulebook lists as usable.
const OTHER: [&str; 3] = ["a", "blockquote", "tg-time"];

/// What the structure check found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MarkupFault {
  Unescaped { character: char },
  Unsupported { tag: String },
  Unclosed { tag: String },
  Unopened { tag: String, open: Option<String> },
  Nested { tag: String, inside: String },
}

/// A fault said in English: which tag, and where it was. A reader wanting it in another language reads the
/// fault's kind and its data instead.
pub fn fault_reason(fault: &MarkupFault) -> String {
  match fault {
    MarkupFault::Unescaped { character } => {
      format!("Structure check: the character {character} outside a tag is not escaped.")
    }
    MarkupFault::Unsupported { tag } => format!("Structure check: the tag <{tag}> is not supported."),
    MarkupFault::Unclosed { tag } => format!("Structure check: the tag <{tag}> is never closed."),
    MarkupFault::Unopened { tag, open: None } => {
      format!("Structure check: the tag </{tag}> closes a tag that was never opened.")
    }
    MarkupFault::Unopened {
      tag,
      open: Some(open),
    } => format!("Structure check: the tag </{tag}> closes while <{open}> is still open."),
    MarkupFault::Nested { tag, inside } => {
      format!("Structure check: the tag <{tag}> stands inside <{inside}>, where it may not.")
    }
  }
}

// Whether a tag may open where the tags already open put it.
fn nesting_fault(tag: &str, stack: &[String]) -> Option<MarkupFault> {
  let nested = |inside: &str| MarkupFault::Nested {
    tag: tag.to_owned(),
    inside: inside.to_owned(),
  };

  if let Some(literal) = stack.iter().find(|open| LITERAL.contains(&open.as_str())) {
    let naming = literal == "pre" && tag == "code" && stack.last().is_some_and(|last| last == "pre");

    return (!naming).then(|| nested(literal));
  }

  if LITERAL.contains(&tag) {
    return stack
      .iter()
      .find(|open| FORMATTING.contains(&open.as_str()) || open.as_str() == "a")
      .map(|holder| nested(holder));
  }

  (tag == "blockquote" && stack.iter().any(|open| open == "blockquote")).then(|| nested("blockquote"))
}

/// The first fault in a piece of markup, or none where it has none: the tags read for what they are and how
/// they pair and nest, and the characters between them for whether the escaping held.
pub fn markup_fault(markup: &str) -> Option<MarkupFault> {
  let mut stack: Vec<String> = vec![];
  let mut index = 0;

  while let Some(character) = markup[index..].chars().next() {
    if character == '&' {
      let Some(found) = ENTITY_AT.find(&markup[index..]) else {
        return Some(MarkupFault::Unescaped { character });
      };

      index += found.end();
      continue;
    }

    if character == '>' {
      return Some(MarkupFault::Unescaped { character });
    }

    if character != '<' {
      index += character.len_utf8();
      continue;
    }

    let Some(token) = TAG_AT.captures(&markup[index..]) else {
      return Some(MarkupFault::Unescaped { character });
    };

    index += token[0].len();

    let tag = token[2].to_lowercase();
    let closing = &token[1] == "/";

    if !FORMATTING.contains(&tag.as_str())
      && !LITERAL.contains(&tag.as_str())
      && !OTHER.contains(&tag.as_str())
    {
      return Some(MarkupFault::Unsupported { tag });
    }

    if !closing && tag == "span" && !SPOILER_CLASS.is_match(&token[3]) {
      return Some(MarkupFault::Unsupported { tag });
    }

    if !closing {
      if let Some(fault) = nesting_fault(&tag, &stack) {
        return Some(fault);
      }

      stack.push(tag);
      continue;
    }

    if stack.last() != Some(&tag) {
      return Some(MarkupFault::Unopened {
        tag,
        open: stack.last().cloned(),
      });
    }

    stack.pop();
  }

  stack.pop().map(|tag| MarkupFault::Unclosed { tag })
}
