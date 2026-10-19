//! A text read for whether it is written in English, two ways. By the policy of the part reading it:
//! the words it reads past and how many Latin letters a Hangul syllable is weighed against, both the
//! part's own, passed in from its own policy. And by script alone, whatever the language: every letter
//! of a script but Latin, written as itself or named by an escape, placed in a comment, a literal or the
//! code by the stretches the caller's reader of the code language marks.

use std::ops::Range;
use std::sync::LazyLock;

use fancy_regex::Regex;

use crate::js::units;

/// The test of whether a text is written in English, as one part's policy declares it: the set-aside words
/// read past, and the Hangul syllables left read against the Latin letters left.
pub struct EnglishTest {
  set_aside: Regex,
  latin_per_hangul: usize,
}

impl EnglishTest {
  /// A test reading past what `set_aside` matches, a regular expression as the part writes it, and weighing
  /// each Hangul syllable left against `latin_per_hangul` Latin letters; at zero, any Hangul left fails it.
  /// A pattern that does not compile gives the compiler's message.
  pub fn try_new(set_aside: &str, latin_per_hangul: usize) -> Result<Self, String> {
    Ok(Self {
      set_aside: Regex::new(set_aside).map_err(|error| error.to_string())?,
      latin_per_hangul,
    })
  }

  /// The test `try_new` makes, panicking on a pattern that does not compile.
  pub fn new(set_aside: &str, latin_per_hangul: usize) -> Self {
    Self::try_new(set_aside, latin_per_hangul).expect("a set-aside pattern of a part's English policy")
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

// A letter of a script of its own other than Latin. A letter of no script of its own, such as the micro
// sign, stands in English as readily as a symbol does, and a mark is no letter.
static NON_ENGLISH: LazyLock<regex::Regex> = LazyLock::new(|| {
  regex::Regex::new(r"[\p{L}--[\p{sc=Latin}\p{sc=Common}\p{sc=Inherited}]]").expect("a class of letters")
});

// A Unicode escape in any of the shapes the code languages share, its code point in hex: a braced `\u`, a
// pattern's braced `\x`, a four-digit `\u` or an eight-digit `\U`. A text may be any code language, a
// comment as much as a literal, and no reader of every one of them exists, so an escape is found by its
// shape.
static ESCAPE: LazyLock<regex::Regex> = LazyLock::new(|| {
  regex::Regex::new(r"\\(?:[ux]\{([0-9A-Fa-f]{1,8})\}|u(?P<four>[0-9A-Fa-f]{4})|U([0-9A-Fa-f]{8}))")
    .expect("a Unicode escape")
});

/// Where a letter that is not English stands in a code text, by the stretches the caller's reader marks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LetterPlace {
  Comment,
  Literal,
  /// In neither a comment nor a literal the caller named.
  Code,
}

impl LetterPlace {
  /// The code the package writes for it: `comment`, `literal` or `code`.
  pub fn code(self) -> &'static str {
    match self {
      Self::Comment => "comment",
      Self::Literal => "literal",
      Self::Code => "code",
    }
  }
}

/// The comments and the literals of a code text, as the caller's own reader of its language marks them,
/// each a range of UTF-16 units. The default names none.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CodeStretches {
  pub comments: Vec<Range<usize>>,
  pub literals: Vec<Range<usize>>,
}

impl CodeStretches {
  // The place of what starts at a UTF-16 index: a comment's before a literal's, the code's in neither.
  fn place_of(&self, at: usize) -> LetterPlace {
    if self.comments.iter().any(|stretch| stretch.contains(&at)) {
      LetterPlace::Comment
    } else if self.literals.iter().any(|stretch| stretch.contains(&at)) {
      LetterPlace::Literal
    } else {
      LetterPlace::Code
    }
  }
}

/// A letter that is not English, where it is written in a text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NonEnglishLetter {
  /// Where the letter, or the whole escape naming it, starts and ends, in UTF-16 units, end exclusive.
  pub start: usize,
  pub end: usize,
  pub letter: char,
  /// Whether an escape names the letter rather than the letter standing as itself.
  pub escaped: bool,
  pub place: LetterPlace,
}

// A letter as the text writes it: its byte range, the letter, and whether an escape names it.
struct Written {
  bytes: Range<usize>,
  letter: char,
  escaped: bool,
}

// One escape of a text: its byte range, the code point it names, and whether it is a four-digit `\u`,
// the one shape that writes half of a surrogate pair.
struct Escape {
  bytes: Range<usize>,
  point: u32,
  four: bool,
}

impl Escape {
  // The letter this escape and the one after it name together, where this is a four-digit high surrogate
  // and the next a four-digit low one written straight after it.
  fn paired(&self, next: &Escape) -> Option<char> {
    let pair = self.four
      && next.four
      && next.bytes.start == self.bytes.end
      && (0xD800..0xDC00).contains(&self.point)
      && (0xDC00..0xE000).contains(&next.point);

    if !pair {
      return None;
    }

    char::from_u32(0x10000 + ((self.point - 0xD800) << 10) + (next.point - 0xDC00))
  }
}

// The letters every escape of a text names, each over the bytes of its escape, a surrogate pair's two
// escapes naming one letter; an escape naming no character, a lone surrogate among them, names none.
fn escaped_letters(text: &str) -> Vec<Written> {
  let escapes: Vec<Escape> = ESCAPE
    .captures_iter(text)
    .filter_map(|escape| {
      let digits = escape.iter().skip(1).flatten().next()?;

      Some(Escape {
        bytes: escape.get_match().range(),
        point: u32::from_str_radix(digits.as_str(), 16).ok()?,
        four: escape.name("four").is_some(),
      })
    })
    .collect();
  let mut letters = vec![];
  let mut index = 0;

  while let Some(escape) = escapes.get(index) {
    let next = escapes.get(index + 1);

    if let Some((letter, next)) = next.and_then(|next| Some((escape.paired(next)?, next))) {
      letters.push(Written {
        bytes: escape.bytes.start..next.bytes.end,
        letter,
        escaped: true,
      });
      index += 2;
    } else {
      letters.extend(char::from_u32(escape.point).map(|letter| Written {
        bytes: escape.bytes.clone(),
        letter,
        escaped: true,
      }));
      index += 1;
    }
  }

  letters
}

// Whether a letter is not English: a letter of a script of its own other than Latin.
fn is_non_english(letter: char) -> bool {
  NON_ENGLISH.is_match(letter.encode_utf8(&mut [0; 4]))
}

/// Every letter of a text that is not English, in the order written: a letter of any script but Latin,
/// standing as itself or named by an escape, placed by `stretches`.
pub fn non_english_letters(text: &str, stretches: &CodeStretches) -> Vec<NonEnglishLetter> {
  let mut written: Vec<Written> = NON_ENGLISH
    .find_iter(text)
    .filter_map(|found| {
      let letter = found.as_str().chars().next()?;

      Some(Written {
        bytes: found.range(),
        letter,
        escaped: false,
      })
    })
    .chain(escaped_letters(text).into_iter().filter(|one| is_non_english(one.letter)))
    .collect();

  written.sort_by_key(|one| one.bytes.start);

  // The UTF-16 count runs on from the letter before, so the text is counted once.
  let mut byte = 0;
  let mut unit = 0;

  written
    .into_iter()
    .map(|one| {
      unit += units(&text[byte..one.bytes.start]);
      byte = one.bytes.start;

      NonEnglishLetter {
        start: unit,
        end: unit + units(&text[one.bytes]),
        letter: one.letter,
        escaped: one.escaped,
        place: stretches.place_of(unit),
      }
    })
    .collect()
}

/// Whether a text holds any letter that is not English, standing as itself or named by an escape.
pub fn holds_non_english(text: &str) -> bool {
  NON_ENGLISH.is_match(text) || escaped_letters(text).iter().any(|one| is_non_english(one.letter))
}
