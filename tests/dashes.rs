use bmo_markup::dashes::*;

#[test]
fn a_name_and_its_explanation_get_a_colon() {
  assert_eq!(without_dashes("The tool \u{2014} it reads files", false), "The tool: it reads files");
}

#[test]
fn an_aside_gets_commas_and_a_range_a_tilde() {
  assert_eq!(
    without_dashes("The run \u{2014} the second one \u{2014} failed.", false),
    "The run, the second one, failed."
  );
  assert_eq!(without_dashes("pages 3\u{2014}5", false), "pages 3~5");
}

#[test]
fn what_is_not_prose_keeps_its_dashes() {
  assert_eq!(without_dashes("`a \u{2014} b`", false), "`a \u{2014} b`");
  assert_eq!(without_dashes("> a \u{2014} b", false), "> a \u{2014} b");
  assert_eq!(without_dashes("a range 3\u{2013}5", false), "a range 3\u{2013}5");
}
