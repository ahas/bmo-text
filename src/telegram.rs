//! What one Telegram message holds, counted in UTF-16 units after the markup is parsed, and plain words too long
//! for one message divided as a client divides a long paste.

use unicode_segmentation::UnicodeSegmentation;

use crate::js::units;

/// One message's text.
pub const TEXT_LIMIT: usize = 4096;

/// A file's caption on an account without Premium, the only size a sender can count on.
pub const CAPTION_LIMIT: usize = 1024;

/// The largest file the Bot API takes, in bytes.
pub const MAX_SEND_FILE_BYTES: usize = 50 * 1024 * 1024;

/// The bot's own Telegram user id, which is the number before the colon of its token, read as JavaScript's
/// `parseInt` reads it: leading white space passed over, a sign, then the digits that stand; nought where there
/// are none.
pub fn bot_id_from_token(token: &str) -> i64 {
  let head = crate::js::trim(token.split(':').next().unwrap_or(""));
  let (sign, digits) = match head.as_bytes().first() {
    Some(b'-') => (-1, &head[1..]),
    Some(b'+') => (1, &head[1..]),
    _ => (1, head),
  };
  let end = digits.find(|character: char| !character.is_ascii_digit()).unwrap_or(digits.len());

  digits[..end].parse::<i64>().map_or(0, |id| sign * id)
}

/// Plain words divided into consecutive messages of at most `limit` UTF-16 units each, the way a Telegram client
/// sends a long paste: each part ends just after the last pause that fits — a blank line, else a line break, else
/// a space — so the pause stays with the words before it, and where none fits, after the last whole character a
/// reader sees as one.
pub fn plain_parts(text: &str, limit: usize) -> Vec<String> {
  let mut parts = vec![];
  let mut rest = text;

  while units(rest) > limit {
    let cut = cut_point(rest, limit);

    parts.push(rest[..cut].to_owned());
    rest = &rest[cut..];
  }

  if !rest.is_empty() || parts.is_empty() {
    parts.push(rest.to_owned());
  }

  parts
}

// The furthest character boundary whose words come within the window.
fn window_end(text: &str, limit: usize) -> usize {
  let mut taken = 0;

  for (at, character) in text.char_indices() {
    taken += character.len_utf16();

    if taken > limit {
      return at;
    }
  }

  text.len()
}

// Where the first part ends, as a byte offset.
fn cut_point(text: &str, limit: usize) -> usize {
  let window = &text[..window_end(text, limit)];

  for pause in ["\n\n", "\n", " "] {
    if let Some(at) = window.rfind(pause).filter(|at| *at > 0) {
      return at + pause.len();
    }
  }

  let mut cut = 0;
  let mut taken = 0;

  for cluster in text.graphemes(true) {
    taken += units(cluster);

    if taken > limit {
      break;
    }

    cut += cluster.len();
  }

  // A single cluster longer than the window, which no real text carries, is divided between characters.
  if cut > 0 {
    cut
  } else {
    window_end(text, limit).max(text.chars().next().map_or(0, char::len_utf8))
  }
}

/// The reading a voice was spoken from, folded under the voice: its first part as the voice's caption, the rest as
/// replies of one message each, every part an expandable quote of its escaped words. A part ends where the next
/// whole character a reader sees as one would not fit, and only a character that alone exceeds a part is
/// divided between its code points; nothing is dropped.
pub fn transcript_parts(text: &str) -> Vec<String> {
  let mut parts = Folds {
    done: vec![],
    part: String::new(),
    limit: CAPTION_LIMIT,
  };

  for cluster in text.graphemes(true) {
    if units(cluster) > parts.limit {
      for point in cluster.chars() {
        parts.append(point.encode_utf8(&mut [0; 4]));
      }
    } else {
      parts.append(cluster);
    }
  }

  if !parts.part.is_empty() {
    parts.done.push(parts.part);
  }

  parts.done.iter().map(|part| crate::format::expandable(part)).collect()
}

// The parts a reading is folded into as it is read: those done, the one being filled, and how far it may run.
struct Folds {
  done: Vec<String>,
  part: String,
  limit: usize,
}

impl Folds {
  fn append(&mut self, piece: &str) {
    if units(&self.part) + units(piece) > self.limit {
      if !self.part.is_empty() {
        self.done.push(std::mem::take(&mut self.part));
      }

      self.limit = if self.done.is_empty() { CAPTION_LIMIT } else { TEXT_LIMIT };
    }

    self.part.push_str(piece);
  }
}
