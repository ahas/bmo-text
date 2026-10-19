//! The ceiling every verb answers under, and the one wording for saying what was left out, the twin of the
//! package's `answer-cap`. The wording is English, since every reader of these answers is an agent; a caller
//! names its unit as a plural noun and says how to ask for the rest as a sentence.
//!
//! A cut nobody is told about is the one shape this must not take: a caller reading a short answer as the
//! whole answer is worse off than one who was refused. So the answer is cut to whole items and the cut is
//! said out loud: how many of the things asked for are in the answer, how many are not, and how to ask for
//! the rest in that verb's own terms.
//!
//! Characters are counted as code points, so a Korean answer is measured in the units a reader counts in.

/// The most characters one verb answers with before it cuts. It stands above what any ordinary answer
/// weighs: the largest shell result an agent session was measured returning is 29,261 characters.
pub const MAX_ANSWER_CHARS: usize = 30_000;

/// How much of a single long value stands before it is cut, where one value is the whole answer.
pub const MAX_VALUE_CHARS: usize = 2_000;

/// What `cap_items` is told: the things counted, as a plural noun, how to ask for the rest, as a sentence,
/// and the ceiling in characters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Said {
  pub unit: String,
  pub how: String,
  pub limit: usize,
}

impl Said {
  /// The words of a cut under the standing ceiling, `MAX_ANSWER_CHARS`.
  pub fn new(unit: &str, how: &str) -> Self {
    Self {
      unit: unit.to_owned(),
      how: how.to_owned(),
      limit: MAX_ANSWER_CHARS,
    }
  }
}

/// What a verb answered with, and what it had to leave out to stay under the ceiling.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Capped<T> {
  /// The things that fit, in the order they were given.
  pub kept: Vec<T>,
  /// How many did not; nought where the whole answer fitted.
  pub left: usize,
  /// The one line saying so, or none where nothing was left out.
  pub note: Option<String>,
}

// A count as the answer writes it, with the thousands mark.
fn count_text(value: usize) -> String {
  let digits = value.to_string();
  let mut grouped = String::new();

  for (at, digit) in digits.chars().enumerate() {
    if at > 0 && (digits.len() - at).is_multiple_of(3) {
      grouped.push(',');
    }

    grouped.push(digit);
  }

  grouped
}

fn chars(text: &str) -> usize {
  text.chars().count()
}

/// The longest run of `items` from the front whose rendering fits under the ceiling, with what was left out
/// named in the caller's own terms.
///
/// `render` is given a prefix and returns exactly what the verb would print for it, so the ceiling is held
/// against the answer itself, separators and all. It is called a handful of times, never once per item.
/// Where even one item is over the ceiling, nothing is kept: an answer cut inside one item would be neither
/// the thing asked for nor valid to parse.
pub fn cap_items<T: Clone>(items: &[T], render: impl Fn(&[T]) -> String, said: &Said) -> Capped<T> {
  let limit = said.limit;

  if chars(&render(items)) <= limit {
    return Capped {
      kept: items.to_vec(),
      left: 0,
      note: None,
    };
  }

  // The largest prefix that fits, found by halving: a longer prefix is a longer answer for every
  // rendering a verb has, so this lands on the largest one that fits.
  let mut fits = 0;
  let mut over = items.len();

  while over - fits > 1 {
    let middle = (fits + over) / 2;

    if chars(&render(&items[..middle])) <= limit {
      fits = middle;
    } else {
      over = middle;
    }
  }

  let left = items.len() - fits;
  let ceiling = count_text(limit);
  let note = if fits == 0 {
    format!(
      "-- one of the {} alone is over the {ceiling}-character ceiling, so nothing was given. {}",
      said.unit, said.how
    )
  } else {
    format!(
      "-- cut at the {ceiling}-character ceiling: {} of {} {}. For the other {}: {}",
      count_text(fits),
      count_text(items.len()),
      said.unit,
      count_text(left),
      said.how
    )
  };

  Capped {
    kept: items[..fits].to_vec(),
    left,
    note: Some(note),
  }
}

/// One long value cut where it stands, with how much was cut said in place of the rest: for the value that
/// is itself the answer, read as prose rather than parsed, so the mark belongs in the line. `limit` counts
/// characters; `MAX_VALUE_CHARS` is the standing one.
pub fn cap_value(text: &str, limit: usize) -> String {
  let held = chars(text);

  if held <= limit {
    return text.to_owned();
  }

  let kept: String = text.chars().take(limit).collect();

  format!("{kept}…(+{} chars)", count_text(held - limit))
}
