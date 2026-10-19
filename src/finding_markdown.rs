//! A search finding's structure lines, turned into the markdown they stand for, the twin of the package's
//! `finding-markdown`. The marks it leaves (a heading per part, a heading per name) are what every reader of
//! a finding counts on standing in the same places. The regular expressions are the package's, with
//! JavaScript's classes spelled out, and those that look around or refer back are run by a backtracking engine
//! as JavaScript runs them.

use std::collections::HashMap;
use std::sync::LazyLock;

use regex::Regex;

use crate::js::{self, DOT, SPACE, units};
use crate::markdown_structure::plain_heading;

/// The parts a label opens. `INTRO`, `BODY` and `CONCLUSION` are the parts an older finding was written in; no
/// search writes them, and the findings that carry them still read.
pub const PART_KEYS: [&str; 7] =
  ["VERDICT", "SUMMARY", "INTRO", "BODY", "CONCLUSION", "UNCONFIRMED", "MORE"];

fn is_part(key: &str) -> bool {
  PART_KEYS.contains(&key)
}

/// The words a reader sees in a drawn finding, the consumer's to choose and read from its own language pack:
/// what each part is called, by its key, and the label of each line an entry draws. `usual` holds `{usual}`
/// where the usual price goes, and `no_rating` is the word a search writes in a review's rating where the
/// source gave none.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FindingWords {
  pub parts: HashMap<String, String>,
  pub address: String,
  pub map: String,
  pub street_view: String,
  pub released: String,
  pub newer: String,
  pub product_page: String,
  pub stock: String,
  pub usual: String,
  pub deal: String,
  pub review: String,
  pub no_rating: String,
  pub menu: String,
  pub dish: String,
  pub price: String,
  pub notes: String,
}

/// The words a finding is drawn with where its consumer passes none: each part called by its key, and every
/// line's label in English.
impl Default for FindingWords {
  fn default() -> Self {
    Self {
      parts: HashMap::new(),
      address: "Address".into(),
      map: "Map".into(),
      street_view: "Street view".into(),
      released: "Released".into(),
      newer: "Newest model".into(),
      product_page: "Product page".into(),
      stock: "Stock".into(),
      usual: "usually {usual}".into(),
      deal: "Deal".into(),
      review: "Review".into(),
      no_rating: "none".into(),
      menu: "Menu".into(),
      dish: "Dish".into(),
      price: "Price".into(),
      notes: "Notes".into(),
    }
  }
}

impl FindingWords {
  // What a reader calls a part: the consumer's word for it, else its key.
  fn part<'a>(&'a self, key: &'a str) -> &'a str {
    self.parts.get(key).map_or(key, String::as_str)
  }
}

static STRUCTURE: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(&format!(
    r"^[ \t]*(TITLE|NAME|VERDICT|SUMMARY|SECTION|INTRO|BODY|CONCLUSION|UNCONFIRMED|MORE|ADDRESS|MAP|ROADVIEW|MENU|RELEASED|NEWER|STOCK|DEAL|IMAGE|REVIEW|SOURCE(?:[ \t]*\[[^\]]*\])?|ANSWERED|TOPIC|KEYWORDS|RULE|KIND|DOCUMENT):[ \t]*({DOT}*?)[ \t]*$"
  ))
  .expect("a structure line")
});

// A footnote mark in a finding's words, [3], or [3†] for a value read only in a search snippet. A bracket
// followed by an address is a link of its own.
static FOOTNOTE: LazyLock<fancy_regex::Regex> =
  LazyLock::new(|| fancy_regex::Regex::new(r"\[([0-9]{1,3}†?)\](?![(:])").expect("a footnote mark"));

/// A finding's footnote marks turned into the in-page links they stand for.
pub fn footnote_marks(line: &str) -> String {
  FOOTNOTE
    .replace_all(line, |found: &fancy_regex::Captures<'_, str>| {
      let mark = &found[1];

      format!("[{mark}](#mark-{})", js::encode_uri_component(mark))
    })
    .into_owned()
}

static FIELD_SEPARATOR: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"[ \t]*::[ \t]*").expect("a separator"));

// What one field line carries, cut at the separator a search writes between its parts.
fn fields(value: &str) -> Vec<&str> {
  FIELD_SEPARATOR.split(value).collect()
}

/// What a picture's line is captioned with where it is shown, out of the fields after its address: what it
/// shows, then its shot and whose picture it is where the line names them. The entry, the third field, is the
/// heading the picture already stands under.
pub fn picture_caption(captioned: &str) -> String {
  let fields = fields(js::trim(captioned));
  let field = |at: usize| fields.get(at).copied().unwrap_or("");

  [field(0), field(2), field(3)]
    .into_iter()
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>()
    .join(" · ")
}

// A heading the search wrote in its own words, and a fence line, outside of which one is read.
static AUTHORED: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(&format!(r"^(#{{1,6}})([ \t]{DOT}*)$")).expect("an authored heading"));
static FENCE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[ \t]*(```|~~~)").expect("a fence"));

// The level the shallowest heading the search wrote itself stands at under structure headings: a name makes
// the third level and a section or a part the fourth, so a heading written inside a section stands under it.
const AUTHORED_FROM: usize = 5;

// The same for a text that draws no structure heading at all, a record or study material written in plain
// markdown: its headings start where a name would, so a book's three levels keep three sizes.
const AUTHORED_ALONE_FROM: usize = 3;

// The parts a result may write with their words on the label's own line, as a brief result does.
fn inline(key: &str) -> bool {
  key == "VERDICT" || key == "SUMMARY"
}

// Whether a structure line is drawn as a heading: a name, a section that says what it holds, or a part label,
// every part but a verdict or a summary that carries its words on the same line.
fn draws_heading(key: &str, value: &str) -> bool {
  key == "TITLE"
    || key == "NAME"
    || (key == "SECTION" && !value.is_empty())
    || (is_part(key) && !(inline(key) && !value.is_empty()))
}

// A structure line's key and value.
fn structure(line: &str) -> Option<(&str, &str)> {
  STRUCTURE.captures(line).map(|found| {
    let (_, [key, value]) = found.extract();

    (key, value)
  })
}

// How far the headings a finding wrote itself move down: from its shallowest to the fifth level under structure
// headings, to the third where there are none. A heading already deeper stays.
fn authored_shift(lines: &[&str]) -> usize {
  let mut fenced = false;
  let mut shallowest = 7;
  let mut structured = false;

  for line in lines {
    if FENCE.is_match(line) {
      fenced = !fenced;
    } else if !fenced {
      shallowest = shallowest.min(AUTHORED.captures(line).map_or(7, |found| found[1].len()));

      if structure(line).is_some_and(|(key, value)| draws_heading(key, value)) {
        structured = true;
      }
    }
  }

  if shallowest == 7 {
    0
  } else {
    (if structured { AUTHORED_FROM } else { AUTHORED_ALONE_FROM }).saturating_sub(shallowest)
  }
}

/// The fold and the gallery a drawn finding leaves for a renderer: `::: <kind>` with an optional label, the
/// lines inside, and a line of `:::` alone. Every other reader of the markdown takes it for a paragraph.
pub static FINDING_BLOCK: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"^::: (fold|gallery)(?: ([^\n]*))?\n([\s\S]*?)\n:::(?:\n|$)").expect("a finding block")
});

/// The map a drawn finding leaves for a renderer: one line naming the place to find.
pub static FINDING_MAP: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^::: map ([^\n]+)(?:\n|$)").expect("a finding map"));

static ITEM: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^[ \t]*===ITEM===[ \t]*$").expect("a divider"));
static IMAGE_LINE: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^[ \t]*IMAGE:").expect("a picture line"));
static BLANK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[ \t]*$").expect("a blank line"));
static SEARCHED: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"/search/([^/?#]+)").expect("a map search"));
static SPACES: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(&format!("[{SPACE}]+")).expect("a run of space"));
static MENU_MARK: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^[ \t]*(?:[-*+]|[0-9]+\.)[ \t]+").expect("a list mark"));
static OWN_BLOCK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[ \t]*:::").expect("a block line"));
static OWN_BLOCK_COLON: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^([ \t]*):").expect("its colon"));
static BRACKETS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[\[\]]").expect("a bracket"));

// The part of an entry a line stands in: an entry opens at a set's divider or at a name line, and what the map
// of its place is found by is read out of the whole of it, wherever in it the lines stand.
#[derive(Clone, Default)]
struct EntryPlace {
  address: Option<String>,
  name: Option<String>,
}

fn entry_places(lines: &[&str]) -> Vec<EntryPlace> {
  let mut places = vec![];
  let mut current = EntryPlace::default();
  let mut opened = vec![];

  for line in lines {
    let found = structure(line);

    if ITEM.is_match(line) || found.is_some_and(|(key, _)| key == "NAME") {
      opened.push(places.len());
      current = EntryPlace::default();
    }

    match found {
      Some(("ADDRESS", value)) if current.address.is_none() => current.address = Some(value.to_owned()),
      Some(("NAME", value)) => current.name = Some(value.to_owned()),
      _ => {}
    }

    places.push(current.clone());
  }

  // An entry is one place to every line of it, so what a later line of it reads is the whole entry's.
  let mut end = places.len();

  for start in opened.into_iter().rev().chain([0]) {
    if start < end {
      let whole = places[end - 1].clone();

      places[start..end].fill(whole);
      end = start;
    }
  }

  places
}

// The words the map of a place is found by: the entry's address, else what its map link searches for, else the
// entry's name. Footnote marks are not part of any of them.
fn map_query(link: &str, place: Option<&EntryPlace>) -> Option<String> {
  let address = place.and_then(|place| place.address.as_deref()).unwrap_or("");
  let address = js::trim(&FOOTNOTE.replace_all(address, "")).to_owned();

  if !address.is_empty() {
    return Some(address);
  }

  if let Some(words) = SEARCHED.captures(link).and_then(|found| js::decode_uri_component(&found[1])) {
    let words = js::trim(&words);

    if !words.is_empty() {
      return Some(words.to_owned());
    }
  }

  let name = js::trim(place.and_then(|place| place.name.as_deref()).unwrap_or(""));

  (!name.is_empty()).then(|| name.to_owned())
}

// A menu as a table: a dish and its price to a row, with a column for notes where a dish has one, and a line
// with no price a group row in bold. A cell keeps a bar the search wrote as a bar.
fn menu_table(items: &[&str], words: &FindingWords) -> Vec<String> {
  let rows: Vec<Vec<String>> = items
    .iter()
    .map(|item| {
      let item = MENU_MARK.replace(item, "");

      fields(js::trim(&item))
        .into_iter()
        .map(|cell| footnote_marks(&cell.replace('|', "\\|")))
        .collect()
    })
    .collect();
  let noted = rows.iter().any(|row| row.len() > 2);
  let cells = |row: &[&str]| format!("| {} |", row.join(" | "));
  let mut table = vec![
    if noted {
      cells(&[&words.dish, &words.price, &words.notes])
    } else {
      cells(&[&words.dish, &words.price])
    },
    format!("| --- | --- |{}", if noted { " --- |" } else { "" }),
  ];

  for row in &rows {
    let dish = row[0].as_str();
    let line = match row.get(1) {
      None if noted => cells(&[&format!("**{dish}**"), "", ""]),
      None => cells(&[&format!("**{dish}**"), ""]),
      Some(price) if noted => cells(&[dish, price, &row[2..].join(" · ")]),
      Some(price) => cells(&[dish, price]),
    };

    table.push(line);
  }

  table
}

// JavaScript's `String.prototype.replace` with a string pattern and a string replacement: the first occurrence
// only, the replacement's `$$`, `$&`, `` $` `` and `$'` read as JavaScript reads them.
fn replace_first(text: &str, pattern: &str, replacement: &str) -> String {
  let Some(at) = text.find(pattern) else {
    return text.to_owned();
  };
  let (before, after) = (&text[..at], &text[at + pattern.len()..]);
  let mut out = String::from(before);
  let mut rest = replacement.chars().peekable();

  while let Some(character) = rest.next() {
    match (character, rest.peek()) {
      ('$', Some('$')) => out.push('$'),
      ('$', Some('&')) => out.push_str(pattern),
      ('$', Some('`')) => out.push_str(before),
      ('$', Some('\'')) => out.push_str(after),
      _ => {
        out.push(character);
        continue;
      }
    }

    rest.next();
  }

  out.push_str(after);
  out
}

/// A finding's structure lines as markdown, drawn with `words`. A picture is a paragraph of its own and a set's
/// divider is a rule, so both are set apart by blank lines. Pictures written together are one gallery, a menu
/// is a fold of its own, and a map line draws the place's map above its link.
pub fn finding_markdown(text: &str, words: &FindingWords) -> String {
  let lines: Vec<&str> = text.split('\n').collect();
  let shift = authored_shift(&lines);
  let places = entry_places(&lines);
  let mut out: Vec<String> = vec![];
  let mut fenced = false;
  let mut at = 0;

  while at < lines.len() {
    let line = lines[at];
    let index = at;

    at += 1;

    if FENCE.is_match(line) {
      fenced = !fenced;
      out.push(line.to_owned());
      continue;
    }

    if fenced {
      out.push(line.to_owned());
      continue;
    }

    // Pictures written one after another, blank lines between them allowed, are one gallery.
    if IMAGE_LINE.is_match(line) {
      let mut run = vec![line];
      let mut next = at;

      while next < lines.len() && (IMAGE_LINE.is_match(lines[next]) || BLANK.is_match(lines[next])) {
        if IMAGE_LINE.is_match(lines[next]) {
          run.push(lines[next]);
          at = next + 1;
        }

        next += 1;
      }

      let pictures: Vec<String> = run
        .iter()
        .map(|picture| {
          let value = structure(picture).map_or("", |(_, value)| value);

          js::trim(&structure_line("IMAGE", value, words).unwrap_or_default()).to_owned()
        })
        .collect();

      out.push(if pictures.len() == 1 {
        format!("\n{}\n", pictures[0])
      } else {
        format!("\n::: gallery\n{}\n:::\n", pictures.join("\n"))
      });
      continue;
    }

    let opening = structure(line);

    // A menu runs to the next structure line, divider, heading or fence, and is folded whole.
    if let Some(("MENU", read)) = opening {
      let mut items = vec![];

      while at < lines.len()
        && !STRUCTURE.is_match(lines[at])
        && !ITEM.is_match(lines[at])
        && !AUTHORED.is_match(lines[at])
        && !FENCE.is_match(lines[at])
      {
        if !BLANK.is_match(lines[at]) {
          items.push(lines[at]);
        }

        at += 1;
      }

      let mut body = vec![];

      if !read.is_empty() {
        body.extend([footnote_marks(read), String::new()]);
      }

      if !items.is_empty() {
        body.extend(menu_table(&items, words));
      }

      if !body.is_empty() {
        out.push(format!("\n::: fold {}\n{}\n:::\n", words.menu, body.join("\n")));
      }

      continue;
    }

    if let Some(("MAP", value)) = opening {
      let link = structure_line("MAP", value, words).unwrap_or_default();

      out.push(match map_query(value, places.get(index)) {
        None => link,
        Some(query) => format!("\n::: map {}\n\n{link}", SPACES.replace_all(&query, " ")),
      });
      continue;
    }

    if let Some(drawn) = line_markdown(line, shift, words) {
      out.push(drawn);
    }
  }

  out.join("\n")
}

// One line of a finding that stands on its own: a divider, a heading the search wrote, a line of its words, or
// a structure line; none for a line that is not drawn.
fn line_markdown(line: &str, shift: usize, words: &FindingWords) -> Option<String> {
  if ITEM.is_match(line) {
    return Some("\n***\n".to_owned());
  }

  // A line the search wrote in the form a finding block takes stays its own words.
  if OWN_BLOCK.is_match(line) {
    return Some(footnote_marks(&OWN_BLOCK_COLON.replace(line, r"${1}\:")));
  }

  if let Some(authored) = AUTHORED.captures(line) {
    let depth = (authored[1].len() + shift).min(6);

    return Some(footnote_marks(&format!("{}{}", "#".repeat(depth), &authored[2])));
  }

  match structure(line) {
    None => Some(footnote_marks(line)),
    Some((key, value)) => structure_line(key, value, words),
  }
}

// A structure line as the markdown it stands for, and none for one that is not drawn.
fn structure_line(key: &str, value: &str, words: &FindingWords) -> Option<String> {
  if is_part(key) && value.is_empty() {
    return Some(format!("\n#### {}\n", words.part(key)));
  }

  let field = |fields: &[&str], at: usize| fields.get(at).copied().unwrap_or("").to_owned();

  Some(match key {
    "TITLE" | "NAME" => format!("\n### {value}\n"),
    // A section of the middle, headed by the name the search gave it for what it holds.
    "SECTION" if value.is_empty() => return None,
    "SECTION" => format!("\n#### {value}\n"),
    "VERDICT" | "SUMMARY" => format!("\n**{}** {}", words.part(key), footnote_marks(value)),
    "INTRO" | "BODY" | "CONCLUSION" | "UNCONFIRMED" | "MORE" => {
      format!("\n#### {}\n\n{}", words.part(key), footnote_marks(value))
    }
    "ADDRESS" => format!("**{}** {value}", words.address),
    "MAP" => format!("[{}](<{value}>)", words.map),
    "ROADVIEW" => format!("[{}](<{value}>)", words.street_view),
    "RELEASED" => format!("**{}** {}", words.released, footnote_marks(value)),
    "NEWER" => format!("**{}** {}", words.newer, footnote_marks(value)),
    // What the product page said of itself, when it was read and where: a selling state with no time on it is
    // a state that was true once, and the page is where a reader checks it again.
    "STOCK" => {
      let parts = fields(value);
      let (state, read, page) = (field(&parts, 0), field(&parts, 1), field(&parts, 2));
      let when = if read.is_empty() { String::new() } else { format!(" · {read}") };
      let place = if page.is_empty() {
        String::new()
      } else {
        format!(" [{}](<{page}>)", words.product_page)
      };

      format!("**{}** {}{when}{place}", words.stock, footnote_marks(&state))
    }
    // An offer against what the thing usually costs, and the search's own word on whether that gap is a gap.
    "DEAL" => {
      let parts = fields(value);
      let (now, usual, real) = (field(&parts, 0), field(&parts, 1), field(&parts, 2));
      let against = if usual.is_empty() {
        String::new()
      } else {
        format!(" ({})", replace_first(&words.usual, "{usual}", &usual))
      };
      let verdict = if real.is_empty() { String::new() } else { format!(" · {}", footnote_marks(&real)) };

      format!("**{}** {}{against}{verdict}", words.deal, footnote_marks(&now))
    }
    "IMAGE" => {
      let parts = fields(value);
      let caption = picture_caption(&parts[1..].join(" :: "));

      format!("\n![{}](<{}>)\n", BRACKETS.replace_all(&caption, r"\$0"), parts[0])
    }
    // One review as it was written: the rating the source gave it, the reviewer's own words and the page they
    // stand on, set as a quotation so it reads as somebody else's words. A field left out does not appear.
    "REVIEW" => {
      let parts = fields(value);
      let (rating, said, page) = (field(&parts, 0), field(&parts, 1), field(&parts, 2));
      let scored = if rating.is_empty() || rating == words.no_rating {
        String::new()
      } else {
        format!("**{rating}** ")
      };
      let place = if page.is_empty() { String::new() } else { format!(" [{}](<{page}>)", words.review) };

      format!("\n> {scored}{}{place}\n", footnote_marks(&said))
    }
    _ => return None,
  })
}

// Lines that are drawn and carry no prose: a table row, a rule, a picture, and a line opening a block of the
// finding's own.
static NOT_PROSE: LazyLock<fancy_regex::Regex> = LazyLock::new(|| {
  fancy_regex::Regex::new(r"^[ \t]*(?:\||!\[|:::|([-*_])(?:[ \t]*\1){2,}[ \t]*$)")
    .expect("a line that is not prose")
});
// What opens a line of a list or a quotation, which the words go on without.
static LEAD: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"^[ \t]*(?:>[ \t]*)*(?:(?:[-*+]|[0-9]{1,3}[.)])[ \t]+)?").expect("a line's lead")
});

// The prose lines of one finding, in order and as plain words. A part label carrying its words on its own line
// gives those words; every other structure line, a heading, a menu and whatever else is drawn rather than said
// gives none. What is folded under `MORE` is past the opening, so a finding's prose ends where its fold begins.
fn prose_lines(text: &str) -> Vec<String> {
  let mut prose = vec![];
  let mut fenced = false;
  let mut menu = false;

  for line in text.split('\n') {
    if FENCE.is_match(line) {
      fenced = !fenced;
      continue;
    }

    let found = if fenced { None } else { structure(line) };

    if menu && found.is_none() && !ITEM.is_match(line) && !AUTHORED.is_match(line) {
      continue;
    }

    menu = found.is_some_and(|(key, _)| key == "MENU");

    if found.is_some_and(|(key, _)| key == "MORE") {
      break;
    }

    if fenced
      || BLANK.is_match(line)
      || ITEM.is_match(line)
      || AUTHORED.is_match(line)
      || NOT_PROSE.is_match(line).unwrap_or(false)
    {
      continue;
    }

    let words = match found {
      None => line,
      Some((key, value)) if is_part(key) => value,
      Some(_) => "",
    };
    let words = LEAD.replace(words, "");
    let words = FOOTNOTE.replace_all(&words, "");
    let plain = plain_heading(&words);
    let plain = SPACES.replace_all(&plain, " ");
    let plain = js::trim(&plain);

    if !plain.is_empty() {
      prose.push(plain.to_owned());
    }
  }

  prose
}

/// The words a set of findings opens with, as one line of plain prose no longer than `limit` UTF-16 units:
/// what a place that shows a document by a few words and draws no markup is given, such as the card a chat app
/// draws for a link to it. A line too long is cut at the last space before the limit and closed with an
/// ellipsis; the cut never parts a character. Empty where the findings say nothing in prose.
pub fn finding_excerpt(texts: &[&str], limit: usize) -> String {
  let mut lines = vec![];
  let mut length = 0;

  for line in texts.iter().flat_map(|text| prose_lines(text)) {
    if length > limit {
      break;
    }

    length += units(&line) + 1;
    lines.push(line);
  }

  let joined = lines.join(" ");
  let total = units(&joined);

  if total <= limit {
    return joined;
  }

  // As JavaScript's `slice(0, limit - 1)`: a limit of nought keeps all but the last unit.
  let keep = if limit == 0 { total - 1 } else { limit - 1 };
  let characters = js::chars(&joined);
  let cut = js::text(&characters[..js::within(&characters, keep)]);
  let kept = match cut.rfind(' ') {
    Some(space) if 2 * js::units_before(&cut, space) > limit => &cut[..space],
    _ => cut.as_str(),
  };

  format!("{}…", js::trim_end(kept))
}
