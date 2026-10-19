mod shared;

use bmo_markup::answer_language::holds_korean;
use serde_json::Value;

#[test]
fn the_shared_cases_hold() {
  shared::hold_all(
    "answer-language",
    &[("holdsKorean", &|args| Value::from(holds_korean(shared::text(&args[0]))))],
  );
}
