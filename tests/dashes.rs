mod shared;

use bmo_text::dashes::*;
use serde_json::Value;

use shared::text;

#[test]
fn the_shared_cases_hold() {
  shared::hold_all(
    "dashes",
    &[("withoutDashes", &|args| {
      Value::from(without_dashes(text(&args[0]), args[1].as_bool().expect("whether it is wrapped")))
    })],
  );
}
