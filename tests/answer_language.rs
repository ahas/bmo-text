use crate::shared;

use bmo_text::answer_language::{holds_korean, text_language};
use serde_json::Value;

#[test]
fn the_shared_cases_hold() {
  shared::hold_all(
    "answer-language",
    &[
      ("holdsKorean", &|args| Value::from(holds_korean(shared::text(&args[0])))),
      ("textLanguage", &|args| Value::from(text_language(shared::text(&args[0])).code())),
    ],
  );
}
