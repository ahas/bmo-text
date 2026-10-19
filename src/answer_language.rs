//! Whether an answer holds Korean, the twin of the package's `answer-language`. Everything the machine says
//! to a chat session is English, so a turn says what language the room is answered in, and every answer is
//! held to it before it goes anywhere.

/// Whether a piece of text holds any Hangul syllable at all.
pub fn holds_korean(text: &str) -> bool {
  text.chars().any(|character| ('가'..='힣').contains(&character))
}
