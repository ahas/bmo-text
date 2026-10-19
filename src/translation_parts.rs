//! How a text is read for a translator, the twin of the package's `translation-parts`: which language it is
//! written in, and the parts it is cut into and joined back from.

use std::sync::LazyLock;

use regex::Regex;

use crate::js::units;
use crate::markdown_structure::{halve_text, markdown_sections};

/// The language a text is written in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextLanguage {
  Ko,
  En,
}

impl TextLanguage {
  /// The code the package writes for it, `ko` or `en`.
  pub fn code(self) -> &'static str {
    match self {
      Self::Ko => "ko",
      Self::En => "en",
    }
  }
}

// A fenced code block, closed by the same fence it opened with; a line starts and ends at any of
// JavaScript's line terminators, as the package's multiline expression reads it.
static FENCED: LazyLock<fancy_regex::Regex> = LazyLock::new(|| {
  let start = r"(?:^|(?<=[\n\r\x{2028}\x{2029}]))";

  fancy_regex::Regex::new(&format!(
    r"{start} {{0,3}}(`{{3,}}|~{{3,}})[^\n]*\n[\s\S]*?{start} {{0,3}}\1[ \t]*(?=[\n\r\x{{2028}}\x{{2029}}]|$)"
  ))
  .expect("a fenced code block")
});
static CODE_SPAN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"`[^`\n]*`").expect("a code span"));

/// Which language a text is written in, by its letters outside code: a text with few Hangul syllables against
/// its Latin letters is English, and every other text is Korean.
pub fn text_language(text: &str) -> TextLanguage {
  let prose = FENCED.replace_all(text, "");
  let prose = CODE_SPAN.replace_all(&prose, "");
  let hangul = prose.chars().filter(|character| ('가'..='힣').contains(character)).count();
  let latin = prose.chars().filter(char::is_ascii_alphabetic).count();

  if hangul * 4 < latin { TextLanguage::En } else { TextLanguage::Ko }
}

// A rough count of the tokens a text costs a model: a Hangul syllable or other non-ASCII UTF-16 unit is most of
// one, an ASCII character about a third.
fn estimated_tokens(text: &str) -> usize {
  let ascii = text.chars().filter(char::is_ascii).count();
  let other = units(text) - ascii;

  // The package's arithmetic, in the same doubles.
  #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation, clippy::cast_sign_loss)]
  let tokens = (other as f64 * 0.8 + ascii as f64 * 0.3).ceil() as usize;

  tokens
}

/// How much of a text one translation part carries, in estimated tokens. The translation of a part has to fit
/// a translator's answer of 2,048 tokens, and a translation into Korean costs more tokens than the English it
/// renders; parts this small let the first one show in seconds.
pub const TRANSLATION_PART_TOKENS: usize = 750;

fn fitted(section: String, budget: usize) -> Vec<String> {
  if estimated_tokens(&section) <= budget {
    return vec![section];
  }

  match halve_text(&section) {
    None => vec![section],
    Some(cut) => {
      let (first, second) = cut.halves;
      let mut parts = fitted(first, budget);

      parts.extend(fitted(second, budget));
      parts
    }
  }
}

/// A text as the parts a translator is given: its sections, the consecutive ones packed together while they fit
/// `budget` estimated tokens, a section too large on its own cut at its paragraphs. Joined back, the parts are
/// the text.
pub fn translation_parts(text: &str, budget: usize) -> Vec<String> {
  let mut parts = vec![];
  let mut current = String::new();

  for piece in markdown_sections(text).into_iter().flat_map(|section| fitted(section, budget)) {
    if !current.is_empty() && estimated_tokens(&format!("{current}{piece}")) > budget {
      parts.push(std::mem::take(&mut current));
    }

    current.push_str(&piece);
  }

  if !current.is_empty() || parts.is_empty() {
    parts.push(current);
  }

  parts
}

/// One part as it stands once read: its translation where there is one, the original where there is none. A
/// translator trims what it hands back, so a part that ended the original's line ends the translation's too,
/// and the parts after it stay blocks of their own.
pub fn shown_part(source: &str, text: Option<&str>) -> String {
  let Some(text) = text else {
    return source.to_owned();
  };

  if source.ends_with('\n') && !text.ends_with('\n') {
    return format!("{text}{}", if source.ends_with("\n\n") { "\n\n" } else { "\n" });
  }

  text.to_owned()
}
