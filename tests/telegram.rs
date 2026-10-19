mod shared;

use bmo_text::telegram::*;
use serde_json::Value;

use shared::text;

#[test]
fn the_shared_cases_hold() {
  shared::hold_all(
    "telegram",
    &[
      ("plainParts", &|args| {
        let limit = usize::try_from(args[1].as_u64().expect("a limit")).expect("a limit");

        Value::from(plain_parts(text(&args[0]), limit))
      }),
      ("transcriptParts", &|args| Value::from(transcript_parts(text(&args[0])))),
      ("botIdFromToken", &|args| Value::from(bot_id_from_token(text(&args[0])))),
    ],
  );
}
