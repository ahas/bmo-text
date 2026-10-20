//! How a text is read for a translator, the twin of the package's `translation-parts`: the parts it is cut
//! into and joined back from. Which language it is written in is `answer_language`'s reading.

use crate::js::units;
use crate::markdown_structure::{halve_text, markdown_sections};

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
