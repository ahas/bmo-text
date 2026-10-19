mod shared;

use bmo_markup::english::*;
use serde_json::Value;

use shared::text;

#[test]
fn the_shared_cases_hold() {
  shared::hold_all(
    "english",
    &[("passes", &|args| {
      let weight = usize::try_from(args[1].as_u64().expect("a weight")).expect("a weight");

      Value::from(EnglishTest::new(text(&args[0]), weight).passes(text(&args[2])))
    })],
  );
}

#[test]
fn the_default_is_the_strictest_test() {
  assert!(EnglishTest::default().passes("plain English"));
  assert!(!EnglishTest::default().passes("almost English 가"));
}
