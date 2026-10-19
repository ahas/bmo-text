mod shared;

use bmo_text::js::{chars, units_of, within};
use bmo_text::web_address::*;
use serde_json::{Value, json};

use shared::text;

// A case's position, in UTF-16 units, as the index of the character it stands before.
fn character_at(text: &[char], units: &Value) -> usize {
  within(text, usize::try_from(units.as_u64().expect("an index")).expect("an index"))
}

// A position in the characters, in UTF-16 units as a case writes it.
fn units_at(text: &[char], at: usize) -> usize {
  units_of(&text[..at])
}

#[test]
fn the_shared_cases_hold() {
  shared::hold_all(
    "web-address",
    &[
      ("addressAsWritten", &|args| Value::from(address_as_written(text(&args[0])))),
      ("isWebAddress", &|args| Value::from(is_web_address(text(&args[0])))),
      ("markdownLinkAt", &|args| {
        let characters = chars(text(&args[0]));

        markdown_link_at(&characters, character_at(&characters, &args[1])).map_or(
          Value::Null,
          |link| json!({ "label": link.label, "url": link.url, "end": units_at(&characters, link.end) }),
        )
      }),
      ("bareAddressAt", &|args| {
        let characters = chars(text(&args[0]));

        bare_address_at(&characters, character_at(&characters, &args[1])).map_or(Value::Null, Value::from)
      }),
      ("gluedAfterAddress", &|args| {
        let characters = chars(text(&args[0]));

        Value::from(glued_after_address(&characters, character_at(&characters, &args[1])))
      }),
      ("webAddresses", &|args| {
        let characters = chars(text(&args[0]));

        web_addresses_in(text(&args[0]))
          .into_iter()
          .map(|found| {
            json!({
              "start": units_at(&characters, found.start),
              "end": units_at(&characters, found.end),
              "url": found.url,
              "label": found.label,
            })
          })
          .collect()
      }),
    ],
  );
}
