//! A text read for whether it is written in English, by the policy of the part reading it: the words it
//! reads past and how many Latin letters a Hangul syllable is weighed against. Both are the part's own,
//! passed in from its own policy.

use regex::Regex;

/// The test of whether a text is written in English, as one part's policy declares it: the set-aside words
/// read past, and the Hangul syllables left read against the Latin letters left.
pub struct EnglishTest {
  set_aside: Regex,
  latin_per_hangul: usize,
}

impl EnglishTest {
  /// A test reading past what `set_aside` matches, a regular expression as the part writes it, and weighing
  /// each Hangul syllable left against `latin_per_hangul` Latin letters; at zero, any Hangul left fails it.
  pub fn new(set_aside: &str, latin_per_hangul: usize) -> Self {
    Self {
      set_aside: Regex::new(set_aside).expect("a set-aside pattern of a part's English policy"),
      latin_per_hangul,
    }
  }

  /// Whether a text is English: with no Latin letters per Hangul syllable, none left at all; otherwise some
  /// Latin letters, and fewer Hangul syllables than one to every so many of them.
  pub fn passes(&self, text: &str) -> bool {
    let prose = self.set_aside.replace_all(text, "");
    let hangul = prose.chars().filter(|character| ('가'..='힣').contains(character)).count();

    if self.latin_per_hangul == 0 {
      return hangul == 0;
    }

    let latin = prose.chars().filter(char::is_ascii_alphabetic).count();

    latin > 0 && hangul * self.latin_per_hangul < latin
  }
}

/// The strictest test: nothing read past, and any Hangul syllable fails it.
impl Default for EnglishTest {
  fn default() -> Self {
    Self::new("", 0)
  }
}
