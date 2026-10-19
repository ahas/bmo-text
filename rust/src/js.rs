//! A text read with JavaScript's character classes, so a reading written in TypeScript keeps its answers:
//! its white space, its trim, its line terminators and its lengths in UTF-16 units.

/// JavaScript's `\s`, as a regular expression's class body.
pub const SPACE: &str =
  r"\t\n\x0B\x0C\r \x{a0}\x{1680}\x{2000}-\x{200a}\x{2028}\x{2029}\x{202f}\x{205f}\x{3000}\x{feff}";

/// JavaScript's `.`: any character but a line terminator, as a regular expression's class.
pub const DOT: &str = r"[^\n\r\x{2028}\x{2029}]";

/// Whether a character is JavaScript's white space.
pub fn is_space(character: char) -> bool {
  matches!(
    character,
    '\t' | '\n' | '\u{b}' | '\u{c}' | '\r' | ' ' | '\u{a0}' | '\u{1680}' | '\u{2000}'
      ..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}'
  )
}

/// A text with JavaScript's white space taken off both ends.
pub fn trim(text: &str) -> &str {
  text.trim_matches(is_space)
}

/// A JavaScript `Number` of a short text, as a verb reads a number it is given: blank is nought, and anything
/// else that is no finite number is none.
pub fn number(text: &str) -> Option<f64> {
  let text = trim(text);

  if text.is_empty() {
    return Some(0.0);
  }

  text.parse::<f64>().ok().filter(|value| value.is_finite())
}

/// A text with JavaScript's white space taken off its end.
pub fn trim_end(text: &str) -> &str {
  text.trim_end_matches(is_space)
}

/// A text's length in UTF-16 units.
pub fn units(text: &str) -> usize {
  text.encode_utf16().count()
}

/// Characters' length in UTF-16 units.
pub fn units_of(characters: &[char]) -> usize {
  characters.iter().map(|character| character.len_utf16()).sum()
}

/// The characters of a text, which every reading here walks by index.
pub fn chars(text: &str) -> Vec<char> {
  text.chars().collect()
}

/// Characters put back into a text.
pub fn text(characters: &[char]) -> String {
  characters.iter().collect()
}

/// How many characters of a text come within a number of UTF-16 units.
pub fn within(characters: &[char], limit: usize) -> usize {
  let mut spent = 0;

  characters
    .iter()
    .take_while(|character| {
      spent += character.len_utf16();
      spent <= limit
    })
    .count()
}

/// The first place a run of characters stands at or after `from`.
pub fn find(characters: &[char], needle: &[char], from: usize) -> Option<usize> {
  (from..=characters.len().saturating_sub(needle.len()))
    .find(|&at| characters.len() >= needle.len() && characters[at..at + needle.len()] == *needle)
}

/// The last place a run of characters starts at or before `before`.
pub fn rfind(characters: &[char], needle: &[char], before: usize) -> Option<usize> {
  if characters.len() < needle.len() {
    return None;
  }

  (0..=before.min(characters.len() - needle.len()))
    .rev()
    .find(|&at| characters[at..at + needle.len()] == *needle)
}

/// A text's character index as a UTF-16 count, given the byte index a regular expression found.
pub fn units_before(text: &str, byte: usize) -> usize {
  units(&text[..byte])
}

// Wyhash's secret, as Zig's standard library writes it.
const WYHASH_SECRET: [u64; 4] = [
  0xa076_1d64_78bd_642f,
  0xe703_7ed1_a0b4_28db,
  0x8ebc_6af0_9c88_c6e3,
  0x5899_65cc_7537_4cc3,
];

fn wyhash_mum(a: u64, b: u64) -> (u64, u64) {
  let product = u128::from(a) * u128::from(b);

  (product as u64, (product >> 64) as u64)
}

fn wyhash_mix(a: u64, b: u64) -> u64 {
  let (low, high) = wyhash_mum(a, b);

  low ^ high
}

fn wyhash_read(bytes: &[u8], at: usize, count: usize) -> u64 {
  bytes[at..at + count]
    .iter()
    .rev()
    .fold(0, |value, byte| (value << 8) | u64::from(*byte))
}

/// What `Bun.hash` answers for a text: Zig's Wyhash of its UTF-8 bytes, seed 0, which the TypeScript side
/// writes a document's version as.
pub fn bun_hash(text: &str) -> u64 {
  let input = text.as_bytes();
  let secret = WYHASH_SECRET;
  let mut state = [0u64; 3];

  state[0] = wyhash_mix(secret[0], secret[1]);
  state[1] = state[0];
  state[2] = state[0];

  let (mut a, mut b);

  if input.len() <= 16 {
    if input.len() >= 4 {
      let end = input.len() - 4;
      let quarter = (input.len() >> 3) << 2;

      a = (wyhash_read(input, 0, 4) << 32) | wyhash_read(input, quarter, 4);
      b = (wyhash_read(input, end, 4) << 32) | wyhash_read(input, end - quarter, 4);
    } else if !input.is_empty() {
      a = (u64::from(input[0]) << 16)
        | (u64::from(input[input.len() >> 1]) << 8)
        | u64::from(input[input.len() - 1]);
      b = 0;
    } else {
      a = 0;
      b = 0;
    }
  } else {
    let mut start = 0;

    if input.len() >= 48 {
      while start + 48 < input.len() {
        for lane in 0..3 {
          let left = wyhash_read(input, start + 16 * lane, 8);
          let right = wyhash_read(input, start + 16 * lane + 8, 8);

          state[lane] = wyhash_mix(left ^ secret[lane + 1], right ^ state[lane]);
        }

        start += 48;
      }

      state[0] ^= state[1] ^ state[2];
    }

    let rest = &input[start..];
    let mut at = 0;

    while at + 16 < rest.len() {
      state[0] = wyhash_mix(wyhash_read(rest, at, 8) ^ secret[1], wyhash_read(rest, at + 8, 8) ^ state[0]);
      at += 16;
    }

    a = wyhash_read(input, input.len() - 16, 8);
    b = wyhash_read(input, input.len() - 8, 8);
  }

  a ^= secret[1];
  b ^= state[0];

  let (low, high) = wyhash_mum(a, b);

  wyhash_mix(low ^ secret[0] ^ input.len() as u64, high ^ secret[1])
}
