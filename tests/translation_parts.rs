mod shared;

use bmo_text::translation_parts::*;
use serde_json::Value;

use shared::text;

#[test]
fn the_shared_cases_hold() {
  shared::hold_all(
    "translation-parts",
    &[
      ("textLanguage", &|args| Value::from(text_language(text(&args[0])).code())),
      ("translationParts", &|args| {
        let budget = usize::try_from(args[1].as_u64().expect("a budget")).expect("a budget");

        Value::from(translation_parts(text(&args[0]), budget))
      }),
      ("shownPart", &|args| Value::from(shown_part(text(&args[0]), args[1].as_str()))),
    ],
  );
}
