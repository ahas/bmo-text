//! What language an answer is written in, the twin of the package's `answer-language`. Everything the machine
//! says to a chat session is English, so a turn says what language the room is answered in, and every answer is
//! held to it before it goes anywhere; a translator is told which way a text goes by the same reading.

use std::sync::LazyLock;

use regex::Regex;

/// Whether a piece of text holds any Hangul syllable at all.
pub fn holds_korean(text: &str) -> bool {
  text.chars().any(hangul_syllable)
}

fn hangul_syllable(character: char) -> bool {
  ('가'..='힣').contains(&character)
}

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
  let hangul = prose.chars().filter(|character| hangul_syllable(*character)).count();
  let latin = prose.chars().filter(char::is_ascii_alphabetic).count();

  if hangul * 4 < latin { TextLanguage::En } else { TextLanguage::Ko }
}
