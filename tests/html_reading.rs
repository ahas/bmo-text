use crate::shared;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use bmo_text::answer_cap::MAX_ANSWER_CHARS;
use bmo_text::html_reading::{HtmlPicture, read_html, read_html_page};
use serde_json::{Value, json};

use shared::text;

#[test]
fn the_shared_cases_hold() {
  shared::hold_all(
    "html-reading",
    &[("readHtml", &|args| {
      let reading = read_html(text(&args[0]), text(&args[1]));
      let picture = |picture: &HtmlPicture| {
        let bytes = STANDARD.encode(&picture.bytes);

        json!({ "name": picture.name, "type": picture.media_type, "bytes": bytes })
      };
      let pictures: Vec<Value> = reading.pictures.iter().map(picture).collect();

      json!({
        "title": reading.title,
        "description": reading.description,
        "lines": reading.lines,
        "pictures": pictures,
      })
    })],
  );
}

// A page of `count` paragraphs, each `length` letters long and opening on its number.
fn paragraphs(count: usize, length: usize) -> String {
  let body: String = (0..count).map(|at| format!("<p>{at:04}{}</p>", "a".repeat(length - 4))).collect();

  format!("<html><head><title>Long page</title></head><body>{body}</body></html>")
}

#[test]
fn a_page_that_fits_is_answered_whole() {
  let page = read_html_page("<title>Short</title><h1>Head</h1><p>One</p><p>Two</p>", "picture", 0);

  assert_eq!(page.title.as_deref(), Some("Short"));
  assert_eq!(
    page.lines,
    read_html("<title>Short</title><h1>Head</h1><p>One</p><p>Two</p>", "picture").lines
  );
  assert!(!page.more);
}

#[test]
fn a_long_page_is_answered_under_the_ceiling_and_read_on_from_where_it_stopped() {
  let html = paragraphs(60, 1_000);
  let whole = read_html(&html, "picture").lines;
  let mut read = vec![];
  let mut from = 0;

  loop {
    let page = read_html_page(&html, "picture", from);

    assert!(page.lines.join("\n").chars().count() <= MAX_ANSWER_CHARS);
    assert!(!page.lines.is_empty());
    from += page.lines.len();
    read.extend(page.lines);

    if !page.more {
      break;
    }
  }

  assert!(from > 1);
  assert_eq!(read, whole);
}

#[test]
fn a_line_over_the_ceiling_alone_is_given_cut_so_the_reading_moves_on() {
  let html = paragraphs(2, MAX_ANSWER_CHARS + 10);
  let page = read_html_page(&html, "picture", 0);

  assert_eq!(page.lines.len(), 1);
  assert!(page.lines[0].starts_with("0000"));
  assert!(page.lines[0].ends_with("…(+10 chars)"));
  assert!(page.more);
}

#[test]
fn a_page_asked_past_its_end_has_no_lines_and_no_more() {
  let page = read_html_page("<p>One</p>", "picture", 5);

  assert!(page.lines.is_empty());
  assert!(!page.more);
}
