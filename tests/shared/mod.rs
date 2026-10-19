//! The cases a module is held to on both sides, read from its file under `cases/` (docs/design.md, 「The
//! twins」). Each function the file names is answered by the test, and every case of it is held. Each test
//! target takes what it needs of this module, so what one leaves unused is no fault.

#![allow(dead_code)]

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde_json::Value;

/// One function of the package's name, answering a case's arguments in the package's JSON shape.
pub type Answer<'a> = (&'a str, &'a dyn Fn(&[Value]) -> Value);

/// Holds every case of `cases/<module>.json`: the functions the file names are exactly those answered, and
/// each case's answer is the one expected. Every case is tried before the test fails, each failure named.
pub fn hold_all(module: &str, answers: &[Answer]) {
  let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("cases").join(format!("{module}.json"));
  let text =
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("{} cannot be read: {error}", path.display()));
  let file: serde_json::Map<String, Value> =
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("{} is not JSON: {error}", path.display()));
  let named: BTreeSet<&str> = file.keys().map(String::as_str).filter(|key| *key != "about").collect();
  let answered: BTreeSet<&str> = answers.iter().map(|(name, _)| *name).collect();

  assert_eq!(named, answered, "the functions of cases/{module}.json are the ones answered");

  let mut failures = vec![];

  for (function, answer) in answers {
    for case in file[*function].as_array().expect("a function's cases are a list") {
      let name = case["name"].as_str().expect("a case is named");
      let args = case["args"].as_array().expect("a case's arguments are a list");
      let got = answer(args);

      if got != case["answer"] {
        failures.push(format!("{function}: {name}\n  expected {}\n  got      {got}", case["answer"]));
      }
    }
  }

  assert!(
    failures.is_empty(),
    "{} case(s) answered differently:\n{}",
    failures.len(),
    failures.join("\n")
  );
}

/// A case's argument as a string.
pub fn text(value: &Value) -> &str {
  value.as_str().expect("a string argument")
}
