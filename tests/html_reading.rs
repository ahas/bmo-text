mod shared;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use bmo_text::html_reading::{HtmlPicture, read_html};
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
